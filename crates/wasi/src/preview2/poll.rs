use crate::preview2::{bindings::io::poll, Table, WasiView};
use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasmtime::component::Resource;

pub type PollableFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
pub type MakeFuture = for<'a> fn(&'a mut dyn Any) -> PollableFuture<'a>;
pub type ClosureFuture = Box<dyn Fn() -> PollableFuture<'static> + Send + Sync + 'static>;

/// A host representation of the `wasi:io/poll.pollable` resource.
///
/// A pollable is not the same thing as a Rust Future: the same pollable may be used to
/// repeatedly check for readiness of a given condition, e.g. if a stream is readable
/// or writable. So, rather than containing a Future, which can only become Ready once, a
/// Pollable contains a way to create a Future in each call to `poll_list`.
pub struct Pollable {
    index: u32,
    make_future: MakeFuture,
    remove_index_on_delete: Option<fn(&mut Table, u32) -> Result<()>>,
}

#[async_trait::async_trait]
pub trait Subscribe: Send + Sync + 'static {
    async fn ready(&mut self);
}

/// Creates a `pollable` resource which is susbcribed to the provided
/// `resource`.
///
/// If `resource` is an owned resource then it will be deleted when the returned
/// resource is deleted. Otherwise the returned resource is considered a "child"
/// of the given `resource` which means that the given resource cannot be
/// deleted while the `pollable` is still alive.
pub fn subscribe<T>(table: &mut Table, resource: Resource<T>) -> Result<Resource<Pollable>>
where
    T: Subscribe,
{
    fn make_future<'a, T>(stream: &'a mut dyn Any) -> PollableFuture<'a>
    where
        T: Subscribe,
    {
        stream.downcast_mut::<T>().unwrap().ready()
    }

    let pollable = Pollable {
        index: resource.rep(),
        remove_index_on_delete: if resource.owned() {
            Some(|table, idx| {
                let resource = Resource::<T>::new_own(idx);
                table.delete_resource(resource)?;
                Ok(())
            })
        } else {
            None
        },
        make_future: make_future::<T>,
    };

    Ok(table.push_child_resource(pollable, &resource)?)
}

#[async_trait::async_trait]
impl<T: WasiView> poll::Host for T {
    async fn poll_list(&mut self, pollables: Vec<Resource<Pollable>>) -> Result<Vec<u32>> {
        type ReadylistIndex = u32;

        let table = self.table_mut();

        let mut table_futures: HashMap<u32, (MakeFuture, Vec<ReadylistIndex>)> = HashMap::new();

        for (ix, p) in pollables.iter().enumerate() {
            let ix: u32 = ix.try_into()?;

            let pollable = table.get_resource(p)?;
            let (_, list) = table_futures
                .entry(pollable.index)
                .or_insert((pollable.make_future, Vec::new()));
            list.push(ix);
        }

        let mut futures: Vec<(PollableFuture<'_>, Vec<ReadylistIndex>)> = Vec::new();
        for (entry, (make_future, readylist_indices)) in table.iter_entries(table_futures) {
            let entry = entry?;
            futures.push((make_future(entry), readylist_indices));
        }

        struct PollList<'a> {
            futures: Vec<(PollableFuture<'a>, Vec<ReadylistIndex>)>,
        }
        impl<'a> Future for PollList<'a> {
            type Output = Vec<u32>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut any_ready = false;
                let mut results = Vec::new();
                for (fut, readylist_indicies) in self.futures.iter_mut() {
                    match fut.as_mut().poll(cx) {
                        Poll::Ready(()) => {
                            results.extend_from_slice(readylist_indicies);
                            any_ready = true;
                        }
                        Poll::Pending => {}
                    }
                }
                if any_ready {
                    Poll::Ready(results)
                } else {
                    Poll::Pending
                }
            }
        }

        Ok(PollList { futures }.await)
    }

    async fn poll_one(&mut self, pollable: Resource<Pollable>) -> Result<()> {
        let table = self.table_mut();

        let pollable = table.get_resource(&pollable)?;
        let ready = (pollable.make_future)(table.get_as_any_mut(pollable.index)?);
        ready.await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: WasiView> crate::preview2::bindings::io::poll::HostPollable for T {
    fn drop(&mut self, pollable: Resource<Pollable>) -> Result<()> {
        let pollable = self.table_mut().delete_resource(pollable)?;
        if let Some(delete) = pollable.remove_index_on_delete {
            delete(self.table_mut(), pollable.index)?;
        }
        Ok(())
    }
}

pub mod sync {
    use crate::preview2::{
        bindings::io::poll as async_poll,
        bindings::sync_io::io::poll::{self, Pollable},
        in_tokio, WasiView,
    };
    use anyhow::Result;
    use wasmtime::component::Resource;

    impl<T: WasiView> poll::Host for T {
        fn poll_list(&mut self, pollables: Vec<Resource<Pollable>>) -> Result<Vec<u32>> {
            in_tokio(async { async_poll::Host::poll_list(self, pollables).await })
        }

        fn poll_one(&mut self, pollable: Resource<Pollable>) -> Result<()> {
            in_tokio(async { async_poll::Host::poll_one(self, pollable).await })
        }
    }

    impl<T: WasiView> crate::preview2::bindings::sync_io::io::poll::HostPollable for T {
        fn drop(&mut self, pollable: Resource<Pollable>) -> Result<()> {
            async_poll::HostPollable::drop(self, pollable)
        }
    }
}
