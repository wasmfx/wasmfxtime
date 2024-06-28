/// Auto-generated bindings for a pre-instantiated version of a
/// copmonent which implements the world `the-lists`.
///
/// This structure is created through [`TheListsPre::new`] which
/// takes a [`InstancePre`](wasmtime::component::InstancePre) that
/// has been created through a [`Linker`](wasmtime::component::Linker).
pub struct TheListsPre<T> {
    instance_pre: wasmtime::component::InstancePre<T>,
    interface0: exports::foo::foo::lists::GuestPre,
}
impl<T> Clone for TheListsPre<T> {
    fn clone(&self) -> Self {
        Self {
            instance_pre: self.instance_pre.clone(),
            interface0: self.interface0.clone(),
        }
    }
}
/// Auto-generated bindings for an instance a component which
/// implements the world `the-lists`.
///
/// This structure is created through either
/// [`TheLists::instantiate`] or by first creating
/// a [`TheListsPre`] followed by using
/// [`TheListsPre::instantiate`].
pub struct TheLists {
    interface0: exports::foo::foo::lists::Guest,
}
const _: () = {
    #[allow(unused_imports)]
    use wasmtime::component::__internal::anyhow;
    impl<_T> TheListsPre<_T> {
        /// Creates a new copy of `TheListsPre` bindings which can then
        /// be used to instantiate into a particular store.
        ///
        /// This method may fail if the compoennt behind `instance_pre`
        /// does not have the required exports.
        pub fn new(
            instance_pre: wasmtime::component::InstancePre<_T>,
        ) -> wasmtime::Result<Self> {
            let _component = instance_pre.component();
            let interface0 = exports::foo::foo::lists::GuestPre::new(_component)?;
            Ok(TheListsPre {
                instance_pre,
                interface0,
            })
        }
        /// Instantiates a new instance of [`TheLists`] within the
        /// `store` provided.
        ///
        /// This function will use `self` as the pre-instantiated
        /// instance to perform instantiation. Afterwards the preloaded
        /// indices in `self` are used to lookup all exports on the
        /// resulting instance.
        pub fn instantiate(
            &self,
            mut store: impl wasmtime::AsContextMut<Data = _T>,
        ) -> wasmtime::Result<TheLists> {
            let mut store = store.as_context_mut();
            let _instance = self.instance_pre.instantiate(&mut store)?;
            let interface0 = self.interface0.load(&mut store, &_instance)?;
            Ok(TheLists { interface0 })
        }
        pub fn engine(&self) -> &wasmtime::Engine {
            self.instance_pre.engine()
        }
        pub fn instance_pre(&self) -> &wasmtime::component::InstancePre<_T> {
            &self.instance_pre
        }
    }
    impl TheLists {
        /// Convenience wrapper around [`TheListsPre::new`] and
        /// [`TheListsPre::instantiate`].
        pub fn instantiate<_T>(
            mut store: impl wasmtime::AsContextMut<Data = _T>,
            component: &wasmtime::component::Component,
            linker: &wasmtime::component::Linker<_T>,
        ) -> wasmtime::Result<TheLists> {
            let pre = linker.instantiate_pre(component)?;
            TheListsPre::new(pre)?.instantiate(store)
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::component::Linker<T>,
            get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
            U: foo::foo::lists::Host,
        {
            foo::foo::lists::add_to_linker(linker, get)?;
            Ok(())
        }
        pub fn foo_foo_lists(&self) -> &exports::foo::foo::lists::Guest {
            &self.interface0
        }
    }
};
pub mod foo {
    pub mod foo {
        #[allow(clippy::all)]
        pub mod lists {
            #[allow(unused_imports)]
            use wasmtime::component::__internal::anyhow;
            #[derive(wasmtime::component::ComponentType)]
            #[derive(wasmtime::component::Lift)]
            #[derive(wasmtime::component::Lower)]
            #[component(record)]
            #[derive(Clone)]
            pub struct OtherRecord {
                #[component(name = "a1")]
                pub a1: u32,
                #[component(name = "a2")]
                pub a2: u64,
                #[component(name = "a3")]
                pub a3: i32,
                #[component(name = "a4")]
                pub a4: i64,
                #[component(name = "b")]
                pub b: wasmtime::component::__internal::String,
                #[component(name = "c")]
                pub c: wasmtime::component::__internal::Vec<u8>,
            }
            impl core::fmt::Debug for OtherRecord {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct("OtherRecord")
                        .field("a1", &self.a1)
                        .field("a2", &self.a2)
                        .field("a3", &self.a3)
                        .field("a4", &self.a4)
                        .field("b", &self.b)
                        .field("c", &self.c)
                        .finish()
                }
            }
            const _: () = {
                assert!(
                    48 == < OtherRecord as wasmtime::component::ComponentType >::SIZE32
                );
                assert!(
                    8 == < OtherRecord as wasmtime::component::ComponentType >::ALIGN32
                );
            };
            #[derive(wasmtime::component::ComponentType)]
            #[derive(wasmtime::component::Lift)]
            #[derive(wasmtime::component::Lower)]
            #[component(record)]
            #[derive(Clone)]
            pub struct SomeRecord {
                #[component(name = "x")]
                pub x: wasmtime::component::__internal::String,
                #[component(name = "y")]
                pub y: OtherRecord,
                #[component(name = "z")]
                pub z: wasmtime::component::__internal::Vec<OtherRecord>,
                #[component(name = "c1")]
                pub c1: u32,
                #[component(name = "c2")]
                pub c2: u64,
                #[component(name = "c3")]
                pub c3: i32,
                #[component(name = "c4")]
                pub c4: i64,
            }
            impl core::fmt::Debug for SomeRecord {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct("SomeRecord")
                        .field("x", &self.x)
                        .field("y", &self.y)
                        .field("z", &self.z)
                        .field("c1", &self.c1)
                        .field("c2", &self.c2)
                        .field("c3", &self.c3)
                        .field("c4", &self.c4)
                        .finish()
                }
            }
            const _: () = {
                assert!(
                    96 == < SomeRecord as wasmtime::component::ComponentType >::SIZE32
                );
                assert!(
                    8 == < SomeRecord as wasmtime::component::ComponentType >::ALIGN32
                );
            };
            #[derive(wasmtime::component::ComponentType)]
            #[derive(wasmtime::component::Lift)]
            #[derive(wasmtime::component::Lower)]
            #[component(variant)]
            #[derive(Clone)]
            pub enum OtherVariant {
                #[component(name = "a")]
                A,
                #[component(name = "b")]
                B(u32),
                #[component(name = "c")]
                C(wasmtime::component::__internal::String),
            }
            impl core::fmt::Debug for OtherVariant {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        OtherVariant::A => f.debug_tuple("OtherVariant::A").finish(),
                        OtherVariant::B(e) => {
                            f.debug_tuple("OtherVariant::B").field(e).finish()
                        }
                        OtherVariant::C(e) => {
                            f.debug_tuple("OtherVariant::C").field(e).finish()
                        }
                    }
                }
            }
            const _: () = {
                assert!(
                    12 == < OtherVariant as wasmtime::component::ComponentType >::SIZE32
                );
                assert!(
                    4 == < OtherVariant as wasmtime::component::ComponentType >::ALIGN32
                );
            };
            #[derive(wasmtime::component::ComponentType)]
            #[derive(wasmtime::component::Lift)]
            #[derive(wasmtime::component::Lower)]
            #[component(variant)]
            #[derive(Clone)]
            pub enum SomeVariant {
                #[component(name = "a")]
                A(wasmtime::component::__internal::String),
                #[component(name = "b")]
                B,
                #[component(name = "c")]
                C(u32),
                #[component(name = "d")]
                D(wasmtime::component::__internal::Vec<OtherVariant>),
            }
            impl core::fmt::Debug for SomeVariant {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        SomeVariant::A(e) => {
                            f.debug_tuple("SomeVariant::A").field(e).finish()
                        }
                        SomeVariant::B => f.debug_tuple("SomeVariant::B").finish(),
                        SomeVariant::C(e) => {
                            f.debug_tuple("SomeVariant::C").field(e).finish()
                        }
                        SomeVariant::D(e) => {
                            f.debug_tuple("SomeVariant::D").field(e).finish()
                        }
                    }
                }
            }
            const _: () = {
                assert!(
                    12 == < SomeVariant as wasmtime::component::ComponentType >::SIZE32
                );
                assert!(
                    4 == < SomeVariant as wasmtime::component::ComponentType >::ALIGN32
                );
            };
            pub type LoadStoreAllSizes = wasmtime::component::__internal::Vec<
                (
                    wasmtime::component::__internal::String,
                    u8,
                    i8,
                    u16,
                    i16,
                    u32,
                    i32,
                    u64,
                    i64,
                    f32,
                    f64,
                    char,
                ),
            >;
            const _: () = {
                assert!(
                    8 == < LoadStoreAllSizes as wasmtime::component::ComponentType
                    >::SIZE32
                );
                assert!(
                    4 == < LoadStoreAllSizes as wasmtime::component::ComponentType
                    >::ALIGN32
                );
            };
            pub trait Host {
                fn list_u8_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<u8>,
                ) -> ();
                fn list_u16_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<u16>,
                ) -> ();
                fn list_u32_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<u32>,
                ) -> ();
                fn list_u64_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<u64>,
                ) -> ();
                fn list_s8_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<i8>,
                ) -> ();
                fn list_s16_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<i16>,
                ) -> ();
                fn list_s32_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<i32>,
                ) -> ();
                fn list_s64_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<i64>,
                ) -> ();
                fn list_float32_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<f32>,
                ) -> ();
                fn list_float64_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<f64>,
                ) -> ();
                fn list_u8_ret(&mut self) -> wasmtime::component::__internal::Vec<u8>;
                fn list_u16_ret(&mut self) -> wasmtime::component::__internal::Vec<u16>;
                fn list_u32_ret(&mut self) -> wasmtime::component::__internal::Vec<u32>;
                fn list_u64_ret(&mut self) -> wasmtime::component::__internal::Vec<u64>;
                fn list_s8_ret(&mut self) -> wasmtime::component::__internal::Vec<i8>;
                fn list_s16_ret(&mut self) -> wasmtime::component::__internal::Vec<i16>;
                fn list_s32_ret(&mut self) -> wasmtime::component::__internal::Vec<i32>;
                fn list_s64_ret(&mut self) -> wasmtime::component::__internal::Vec<i64>;
                fn list_float32_ret(
                    &mut self,
                ) -> wasmtime::component::__internal::Vec<f32>;
                fn list_float64_ret(
                    &mut self,
                ) -> wasmtime::component::__internal::Vec<f64>;
                fn tuple_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<(u8, i8)>,
                ) -> wasmtime::component::__internal::Vec<(i64, u32)>;
                fn string_list_arg(
                    &mut self,
                    a: wasmtime::component::__internal::Vec<
                        wasmtime::component::__internal::String,
                    >,
                ) -> ();
                fn string_list_ret(
                    &mut self,
                ) -> wasmtime::component::__internal::Vec<
                    wasmtime::component::__internal::String,
                >;
                fn tuple_string_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<
                        (u8, wasmtime::component::__internal::String),
                    >,
                ) -> wasmtime::component::__internal::Vec<
                    (wasmtime::component::__internal::String, u8),
                >;
                fn string_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<
                        wasmtime::component::__internal::String,
                    >,
                ) -> wasmtime::component::__internal::Vec<
                    wasmtime::component::__internal::String,
                >;
                fn record_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<SomeRecord>,
                ) -> wasmtime::component::__internal::Vec<OtherRecord>;
                fn record_list_reverse(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<OtherRecord>,
                ) -> wasmtime::component::__internal::Vec<SomeRecord>;
                fn variant_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<SomeVariant>,
                ) -> wasmtime::component::__internal::Vec<OtherVariant>;
                fn load_store_everything(
                    &mut self,
                    a: LoadStoreAllSizes,
                ) -> LoadStoreAllSizes;
            }
            pub trait GetHost<
                T,
            >: Fn(T) -> <Self as GetHost<T>>::Host + Send + Sync + Copy + 'static {
                type Host: Host;
            }
            impl<F, T, O> GetHost<T> for F
            where
                F: Fn(T) -> O + Send + Sync + Copy + 'static,
                O: Host,
            {
                type Host = O;
            }
            pub fn add_to_linker_get_host<T>(
                linker: &mut wasmtime::component::Linker<T>,
                host_getter: impl for<'a> GetHost<&'a mut T>,
            ) -> wasmtime::Result<()> {
                let mut inst = linker.instance("foo:foo/lists")?;
                inst.func_wrap(
                    "list-u8-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<u8>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_u8_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-u16-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<u16>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_u16_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-u32-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<u32>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_u32_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-u64-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<u64>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_u64_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-s8-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<i8>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_s8_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-s16-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<i16>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_s16_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-s32-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<i32>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_s32_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-s64-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<i64>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_s64_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-float32-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<f32>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_float32_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-float64-param",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<f64>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_float64_param(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "list-u8-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_u8_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-u16-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_u16_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-u32-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_u32_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-u64-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_u64_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-s8-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_s8_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-s16-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_s16_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-s32-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_s32_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-s64-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_s64_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-float32-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_float32_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "list-float64-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::list_float64_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "tuple-list",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<(u8, i8)>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::tuple_list(host, arg0);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "string-list-arg",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (
                            arg0,
                        ): (
                            wasmtime::component::__internal::Vec<
                                wasmtime::component::__internal::String,
                            >,
                        )|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::string_list_arg(host, arg0);
                        Ok(r)
                    },
                )?;
                inst.func_wrap(
                    "string-list-ret",
                    move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::string_list_ret(host);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "tuple-string-list",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (
                            arg0,
                        ): (
                            wasmtime::component::__internal::Vec<
                                (u8, wasmtime::component::__internal::String),
                            >,
                        )|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::tuple_string_list(host, arg0);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "string-list",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (
                            arg0,
                        ): (
                            wasmtime::component::__internal::Vec<
                                wasmtime::component::__internal::String,
                            >,
                        )|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::string_list(host, arg0);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "record-list",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<SomeRecord>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::record_list(host, arg0);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "record-list-reverse",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<OtherRecord>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::record_list_reverse(host, arg0);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "variant-list",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (wasmtime::component::__internal::Vec<SomeVariant>,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::variant_list(host, arg0);
                        Ok((r,))
                    },
                )?;
                inst.func_wrap(
                    "load-store-everything",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (arg0,): (LoadStoreAllSizes,)|
                    {
                        let host = &mut host_getter(caller.data_mut());
                        let r = Host::load_store_everything(host, arg0);
                        Ok((r,))
                    },
                )?;
                Ok(())
            }
            pub fn add_to_linker<T, U>(
                linker: &mut wasmtime::component::Linker<T>,
                get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
            ) -> wasmtime::Result<()>
            where
                U: Host,
            {
                add_to_linker_get_host(linker, get)
            }
            impl<_T: Host + ?Sized> Host for &mut _T {
                fn list_u8_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<u8>,
                ) -> () {
                    Host::list_u8_param(*self, x)
                }
                fn list_u16_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<u16>,
                ) -> () {
                    Host::list_u16_param(*self, x)
                }
                fn list_u32_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<u32>,
                ) -> () {
                    Host::list_u32_param(*self, x)
                }
                fn list_u64_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<u64>,
                ) -> () {
                    Host::list_u64_param(*self, x)
                }
                fn list_s8_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<i8>,
                ) -> () {
                    Host::list_s8_param(*self, x)
                }
                fn list_s16_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<i16>,
                ) -> () {
                    Host::list_s16_param(*self, x)
                }
                fn list_s32_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<i32>,
                ) -> () {
                    Host::list_s32_param(*self, x)
                }
                fn list_s64_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<i64>,
                ) -> () {
                    Host::list_s64_param(*self, x)
                }
                fn list_float32_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<f32>,
                ) -> () {
                    Host::list_float32_param(*self, x)
                }
                fn list_float64_param(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<f64>,
                ) -> () {
                    Host::list_float64_param(*self, x)
                }
                fn list_u8_ret(&mut self) -> wasmtime::component::__internal::Vec<u8> {
                    Host::list_u8_ret(*self)
                }
                fn list_u16_ret(&mut self) -> wasmtime::component::__internal::Vec<u16> {
                    Host::list_u16_ret(*self)
                }
                fn list_u32_ret(&mut self) -> wasmtime::component::__internal::Vec<u32> {
                    Host::list_u32_ret(*self)
                }
                fn list_u64_ret(&mut self) -> wasmtime::component::__internal::Vec<u64> {
                    Host::list_u64_ret(*self)
                }
                fn list_s8_ret(&mut self) -> wasmtime::component::__internal::Vec<i8> {
                    Host::list_s8_ret(*self)
                }
                fn list_s16_ret(&mut self) -> wasmtime::component::__internal::Vec<i16> {
                    Host::list_s16_ret(*self)
                }
                fn list_s32_ret(&mut self) -> wasmtime::component::__internal::Vec<i32> {
                    Host::list_s32_ret(*self)
                }
                fn list_s64_ret(&mut self) -> wasmtime::component::__internal::Vec<i64> {
                    Host::list_s64_ret(*self)
                }
                fn list_float32_ret(
                    &mut self,
                ) -> wasmtime::component::__internal::Vec<f32> {
                    Host::list_float32_ret(*self)
                }
                fn list_float64_ret(
                    &mut self,
                ) -> wasmtime::component::__internal::Vec<f64> {
                    Host::list_float64_ret(*self)
                }
                fn tuple_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<(u8, i8)>,
                ) -> wasmtime::component::__internal::Vec<(i64, u32)> {
                    Host::tuple_list(*self, x)
                }
                fn string_list_arg(
                    &mut self,
                    a: wasmtime::component::__internal::Vec<
                        wasmtime::component::__internal::String,
                    >,
                ) -> () {
                    Host::string_list_arg(*self, a)
                }
                fn string_list_ret(
                    &mut self,
                ) -> wasmtime::component::__internal::Vec<
                    wasmtime::component::__internal::String,
                > {
                    Host::string_list_ret(*self)
                }
                fn tuple_string_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<
                        (u8, wasmtime::component::__internal::String),
                    >,
                ) -> wasmtime::component::__internal::Vec<
                    (wasmtime::component::__internal::String, u8),
                > {
                    Host::tuple_string_list(*self, x)
                }
                fn string_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<
                        wasmtime::component::__internal::String,
                    >,
                ) -> wasmtime::component::__internal::Vec<
                    wasmtime::component::__internal::String,
                > {
                    Host::string_list(*self, x)
                }
                fn record_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<SomeRecord>,
                ) -> wasmtime::component::__internal::Vec<OtherRecord> {
                    Host::record_list(*self, x)
                }
                fn record_list_reverse(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<OtherRecord>,
                ) -> wasmtime::component::__internal::Vec<SomeRecord> {
                    Host::record_list_reverse(*self, x)
                }
                fn variant_list(
                    &mut self,
                    x: wasmtime::component::__internal::Vec<SomeVariant>,
                ) -> wasmtime::component::__internal::Vec<OtherVariant> {
                    Host::variant_list(*self, x)
                }
                fn load_store_everything(
                    &mut self,
                    a: LoadStoreAllSizes,
                ) -> LoadStoreAllSizes {
                    Host::load_store_everything(*self, a)
                }
            }
        }
    }
}
pub mod exports {
    pub mod foo {
        pub mod foo {
            #[allow(clippy::all)]
            pub mod lists {
                #[allow(unused_imports)]
                use wasmtime::component::__internal::anyhow;
                #[derive(wasmtime::component::ComponentType)]
                #[derive(wasmtime::component::Lift)]
                #[derive(wasmtime::component::Lower)]
                #[component(record)]
                #[derive(Clone)]
                pub struct OtherRecord {
                    #[component(name = "a1")]
                    pub a1: u32,
                    #[component(name = "a2")]
                    pub a2: u64,
                    #[component(name = "a3")]
                    pub a3: i32,
                    #[component(name = "a4")]
                    pub a4: i64,
                    #[component(name = "b")]
                    pub b: wasmtime::component::__internal::String,
                    #[component(name = "c")]
                    pub c: wasmtime::component::__internal::Vec<u8>,
                }
                impl core::fmt::Debug for OtherRecord {
                    fn fmt(
                        &self,
                        f: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        f.debug_struct("OtherRecord")
                            .field("a1", &self.a1)
                            .field("a2", &self.a2)
                            .field("a3", &self.a3)
                            .field("a4", &self.a4)
                            .field("b", &self.b)
                            .field("c", &self.c)
                            .finish()
                    }
                }
                const _: () = {
                    assert!(
                        48 == < OtherRecord as wasmtime::component::ComponentType
                        >::SIZE32
                    );
                    assert!(
                        8 == < OtherRecord as wasmtime::component::ComponentType
                        >::ALIGN32
                    );
                };
                #[derive(wasmtime::component::ComponentType)]
                #[derive(wasmtime::component::Lift)]
                #[derive(wasmtime::component::Lower)]
                #[component(record)]
                #[derive(Clone)]
                pub struct SomeRecord {
                    #[component(name = "x")]
                    pub x: wasmtime::component::__internal::String,
                    #[component(name = "y")]
                    pub y: OtherRecord,
                    #[component(name = "z")]
                    pub z: wasmtime::component::__internal::Vec<OtherRecord>,
                    #[component(name = "c1")]
                    pub c1: u32,
                    #[component(name = "c2")]
                    pub c2: u64,
                    #[component(name = "c3")]
                    pub c3: i32,
                    #[component(name = "c4")]
                    pub c4: i64,
                }
                impl core::fmt::Debug for SomeRecord {
                    fn fmt(
                        &self,
                        f: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        f.debug_struct("SomeRecord")
                            .field("x", &self.x)
                            .field("y", &self.y)
                            .field("z", &self.z)
                            .field("c1", &self.c1)
                            .field("c2", &self.c2)
                            .field("c3", &self.c3)
                            .field("c4", &self.c4)
                            .finish()
                    }
                }
                const _: () = {
                    assert!(
                        96 == < SomeRecord as wasmtime::component::ComponentType
                        >::SIZE32
                    );
                    assert!(
                        8 == < SomeRecord as wasmtime::component::ComponentType
                        >::ALIGN32
                    );
                };
                #[derive(wasmtime::component::ComponentType)]
                #[derive(wasmtime::component::Lift)]
                #[derive(wasmtime::component::Lower)]
                #[component(variant)]
                #[derive(Clone)]
                pub enum OtherVariant {
                    #[component(name = "a")]
                    A,
                    #[component(name = "b")]
                    B(u32),
                    #[component(name = "c")]
                    C(wasmtime::component::__internal::String),
                }
                impl core::fmt::Debug for OtherVariant {
                    fn fmt(
                        &self,
                        f: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        match self {
                            OtherVariant::A => f.debug_tuple("OtherVariant::A").finish(),
                            OtherVariant::B(e) => {
                                f.debug_tuple("OtherVariant::B").field(e).finish()
                            }
                            OtherVariant::C(e) => {
                                f.debug_tuple("OtherVariant::C").field(e).finish()
                            }
                        }
                    }
                }
                const _: () = {
                    assert!(
                        12 == < OtherVariant as wasmtime::component::ComponentType
                        >::SIZE32
                    );
                    assert!(
                        4 == < OtherVariant as wasmtime::component::ComponentType
                        >::ALIGN32
                    );
                };
                #[derive(wasmtime::component::ComponentType)]
                #[derive(wasmtime::component::Lift)]
                #[derive(wasmtime::component::Lower)]
                #[component(variant)]
                #[derive(Clone)]
                pub enum SomeVariant {
                    #[component(name = "a")]
                    A(wasmtime::component::__internal::String),
                    #[component(name = "b")]
                    B,
                    #[component(name = "c")]
                    C(u32),
                    #[component(name = "d")]
                    D(wasmtime::component::__internal::Vec<OtherVariant>),
                }
                impl core::fmt::Debug for SomeVariant {
                    fn fmt(
                        &self,
                        f: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        match self {
                            SomeVariant::A(e) => {
                                f.debug_tuple("SomeVariant::A").field(e).finish()
                            }
                            SomeVariant::B => f.debug_tuple("SomeVariant::B").finish(),
                            SomeVariant::C(e) => {
                                f.debug_tuple("SomeVariant::C").field(e).finish()
                            }
                            SomeVariant::D(e) => {
                                f.debug_tuple("SomeVariant::D").field(e).finish()
                            }
                        }
                    }
                }
                const _: () = {
                    assert!(
                        12 == < SomeVariant as wasmtime::component::ComponentType
                        >::SIZE32
                    );
                    assert!(
                        4 == < SomeVariant as wasmtime::component::ComponentType
                        >::ALIGN32
                    );
                };
                pub type LoadStoreAllSizes = wasmtime::component::__internal::Vec<
                    (
                        wasmtime::component::__internal::String,
                        u8,
                        i8,
                        u16,
                        i16,
                        u32,
                        i32,
                        u64,
                        i64,
                        f32,
                        f64,
                        char,
                    ),
                >;
                const _: () = {
                    assert!(
                        8 == < LoadStoreAllSizes as wasmtime::component::ComponentType
                        >::SIZE32
                    );
                    assert!(
                        4 == < LoadStoreAllSizes as wasmtime::component::ComponentType
                        >::ALIGN32
                    );
                };
                pub struct Guest {
                    list_u8_param: wasmtime::component::Func,
                    list_u16_param: wasmtime::component::Func,
                    list_u32_param: wasmtime::component::Func,
                    list_u64_param: wasmtime::component::Func,
                    list_s8_param: wasmtime::component::Func,
                    list_s16_param: wasmtime::component::Func,
                    list_s32_param: wasmtime::component::Func,
                    list_s64_param: wasmtime::component::Func,
                    list_float32_param: wasmtime::component::Func,
                    list_float64_param: wasmtime::component::Func,
                    list_u8_ret: wasmtime::component::Func,
                    list_u16_ret: wasmtime::component::Func,
                    list_u32_ret: wasmtime::component::Func,
                    list_u64_ret: wasmtime::component::Func,
                    list_s8_ret: wasmtime::component::Func,
                    list_s16_ret: wasmtime::component::Func,
                    list_s32_ret: wasmtime::component::Func,
                    list_s64_ret: wasmtime::component::Func,
                    list_float32_ret: wasmtime::component::Func,
                    list_float64_ret: wasmtime::component::Func,
                    tuple_list: wasmtime::component::Func,
                    string_list_arg: wasmtime::component::Func,
                    string_list_ret: wasmtime::component::Func,
                    tuple_string_list: wasmtime::component::Func,
                    string_list: wasmtime::component::Func,
                    record_list: wasmtime::component::Func,
                    record_list_reverse: wasmtime::component::Func,
                    variant_list: wasmtime::component::Func,
                    load_store_everything: wasmtime::component::Func,
                }
                #[derive(Clone)]
                pub struct GuestPre {
                    list_u8_param: wasmtime::component::ComponentExportIndex,
                    list_u16_param: wasmtime::component::ComponentExportIndex,
                    list_u32_param: wasmtime::component::ComponentExportIndex,
                    list_u64_param: wasmtime::component::ComponentExportIndex,
                    list_s8_param: wasmtime::component::ComponentExportIndex,
                    list_s16_param: wasmtime::component::ComponentExportIndex,
                    list_s32_param: wasmtime::component::ComponentExportIndex,
                    list_s64_param: wasmtime::component::ComponentExportIndex,
                    list_float32_param: wasmtime::component::ComponentExportIndex,
                    list_float64_param: wasmtime::component::ComponentExportIndex,
                    list_u8_ret: wasmtime::component::ComponentExportIndex,
                    list_u16_ret: wasmtime::component::ComponentExportIndex,
                    list_u32_ret: wasmtime::component::ComponentExportIndex,
                    list_u64_ret: wasmtime::component::ComponentExportIndex,
                    list_s8_ret: wasmtime::component::ComponentExportIndex,
                    list_s16_ret: wasmtime::component::ComponentExportIndex,
                    list_s32_ret: wasmtime::component::ComponentExportIndex,
                    list_s64_ret: wasmtime::component::ComponentExportIndex,
                    list_float32_ret: wasmtime::component::ComponentExportIndex,
                    list_float64_ret: wasmtime::component::ComponentExportIndex,
                    tuple_list: wasmtime::component::ComponentExportIndex,
                    string_list_arg: wasmtime::component::ComponentExportIndex,
                    string_list_ret: wasmtime::component::ComponentExportIndex,
                    tuple_string_list: wasmtime::component::ComponentExportIndex,
                    string_list: wasmtime::component::ComponentExportIndex,
                    record_list: wasmtime::component::ComponentExportIndex,
                    record_list_reverse: wasmtime::component::ComponentExportIndex,
                    variant_list: wasmtime::component::ComponentExportIndex,
                    load_store_everything: wasmtime::component::ComponentExportIndex,
                }
                impl GuestPre {
                    pub fn new(
                        component: &wasmtime::component::Component,
                    ) -> wasmtime::Result<GuestPre> {
                        let _component = component;
                        let (_, instance) = component
                            .export_index(None, "foo:foo/lists")
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "no exported instance named `foo:foo/lists`"
                                )
                            })?;
                        let _lookup = |name: &str| {
                            _component
                                .export_index(Some(&instance), name)
                                .map(|p| p.1)
                                .ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "instance export `foo:foo/lists` does \
                not have export `{name}`"
                                    )
                                })
                        };
                        let list_u8_param = _lookup("list-u8-param")?;
                        let list_u16_param = _lookup("list-u16-param")?;
                        let list_u32_param = _lookup("list-u32-param")?;
                        let list_u64_param = _lookup("list-u64-param")?;
                        let list_s8_param = _lookup("list-s8-param")?;
                        let list_s16_param = _lookup("list-s16-param")?;
                        let list_s32_param = _lookup("list-s32-param")?;
                        let list_s64_param = _lookup("list-s64-param")?;
                        let list_float32_param = _lookup("list-float32-param")?;
                        let list_float64_param = _lookup("list-float64-param")?;
                        let list_u8_ret = _lookup("list-u8-ret")?;
                        let list_u16_ret = _lookup("list-u16-ret")?;
                        let list_u32_ret = _lookup("list-u32-ret")?;
                        let list_u64_ret = _lookup("list-u64-ret")?;
                        let list_s8_ret = _lookup("list-s8-ret")?;
                        let list_s16_ret = _lookup("list-s16-ret")?;
                        let list_s32_ret = _lookup("list-s32-ret")?;
                        let list_s64_ret = _lookup("list-s64-ret")?;
                        let list_float32_ret = _lookup("list-float32-ret")?;
                        let list_float64_ret = _lookup("list-float64-ret")?;
                        let tuple_list = _lookup("tuple-list")?;
                        let string_list_arg = _lookup("string-list-arg")?;
                        let string_list_ret = _lookup("string-list-ret")?;
                        let tuple_string_list = _lookup("tuple-string-list")?;
                        let string_list = _lookup("string-list")?;
                        let record_list = _lookup("record-list")?;
                        let record_list_reverse = _lookup("record-list-reverse")?;
                        let variant_list = _lookup("variant-list")?;
                        let load_store_everything = _lookup("load-store-everything")?;
                        Ok(GuestPre {
                            list_u8_param,
                            list_u16_param,
                            list_u32_param,
                            list_u64_param,
                            list_s8_param,
                            list_s16_param,
                            list_s32_param,
                            list_s64_param,
                            list_float32_param,
                            list_float64_param,
                            list_u8_ret,
                            list_u16_ret,
                            list_u32_ret,
                            list_u64_ret,
                            list_s8_ret,
                            list_s16_ret,
                            list_s32_ret,
                            list_s64_ret,
                            list_float32_ret,
                            list_float64_ret,
                            tuple_list,
                            string_list_arg,
                            string_list_ret,
                            tuple_string_list,
                            string_list,
                            record_list,
                            record_list_reverse,
                            variant_list,
                            load_store_everything,
                        })
                    }
                    pub fn load(
                        &self,
                        mut store: impl wasmtime::AsContextMut,
                        instance: &wasmtime::component::Instance,
                    ) -> wasmtime::Result<Guest> {
                        let mut store = store.as_context_mut();
                        let _ = &mut store;
                        let _instance = instance;
                        let list_u8_param = *_instance
                            .get_typed_func::<
                                (&[u8],),
                                (),
                            >(&mut store, &self.list_u8_param)?
                            .func();
                        let list_u16_param = *_instance
                            .get_typed_func::<
                                (&[u16],),
                                (),
                            >(&mut store, &self.list_u16_param)?
                            .func();
                        let list_u32_param = *_instance
                            .get_typed_func::<
                                (&[u32],),
                                (),
                            >(&mut store, &self.list_u32_param)?
                            .func();
                        let list_u64_param = *_instance
                            .get_typed_func::<
                                (&[u64],),
                                (),
                            >(&mut store, &self.list_u64_param)?
                            .func();
                        let list_s8_param = *_instance
                            .get_typed_func::<
                                (&[i8],),
                                (),
                            >(&mut store, &self.list_s8_param)?
                            .func();
                        let list_s16_param = *_instance
                            .get_typed_func::<
                                (&[i16],),
                                (),
                            >(&mut store, &self.list_s16_param)?
                            .func();
                        let list_s32_param = *_instance
                            .get_typed_func::<
                                (&[i32],),
                                (),
                            >(&mut store, &self.list_s32_param)?
                            .func();
                        let list_s64_param = *_instance
                            .get_typed_func::<
                                (&[i64],),
                                (),
                            >(&mut store, &self.list_s64_param)?
                            .func();
                        let list_float32_param = *_instance
                            .get_typed_func::<
                                (&[f32],),
                                (),
                            >(&mut store, &self.list_float32_param)?
                            .func();
                        let list_float64_param = *_instance
                            .get_typed_func::<
                                (&[f64],),
                                (),
                            >(&mut store, &self.list_float64_param)?
                            .func();
                        let list_u8_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<u8>,),
                            >(&mut store, &self.list_u8_ret)?
                            .func();
                        let list_u16_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<u16>,),
                            >(&mut store, &self.list_u16_ret)?
                            .func();
                        let list_u32_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<u32>,),
                            >(&mut store, &self.list_u32_ret)?
                            .func();
                        let list_u64_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<u64>,),
                            >(&mut store, &self.list_u64_ret)?
                            .func();
                        let list_s8_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<i8>,),
                            >(&mut store, &self.list_s8_ret)?
                            .func();
                        let list_s16_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<i16>,),
                            >(&mut store, &self.list_s16_ret)?
                            .func();
                        let list_s32_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<i32>,),
                            >(&mut store, &self.list_s32_ret)?
                            .func();
                        let list_s64_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<i64>,),
                            >(&mut store, &self.list_s64_ret)?
                            .func();
                        let list_float32_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<f32>,),
                            >(&mut store, &self.list_float32_ret)?
                            .func();
                        let list_float64_ret = *_instance
                            .get_typed_func::<
                                (),
                                (wasmtime::component::__internal::Vec<f64>,),
                            >(&mut store, &self.list_float64_ret)?
                            .func();
                        let tuple_list = *_instance
                            .get_typed_func::<
                                (&[(u8, i8)],),
                                (wasmtime::component::__internal::Vec<(i64, u32)>,),
                            >(&mut store, &self.tuple_list)?
                            .func();
                        let string_list_arg = *_instance
                            .get_typed_func::<
                                (&[wasmtime::component::__internal::String],),
                                (),
                            >(&mut store, &self.string_list_arg)?
                            .func();
                        let string_list_ret = *_instance
                            .get_typed_func::<
                                (),
                                (
                                    wasmtime::component::__internal::Vec<
                                        wasmtime::component::__internal::String,
                                    >,
                                ),
                            >(&mut store, &self.string_list_ret)?
                            .func();
                        let tuple_string_list = *_instance
                            .get_typed_func::<
                                (&[(u8, wasmtime::component::__internal::String)],),
                                (
                                    wasmtime::component::__internal::Vec<
                                        (wasmtime::component::__internal::String, u8),
                                    >,
                                ),
                            >(&mut store, &self.tuple_string_list)?
                            .func();
                        let string_list = *_instance
                            .get_typed_func::<
                                (&[wasmtime::component::__internal::String],),
                                (
                                    wasmtime::component::__internal::Vec<
                                        wasmtime::component::__internal::String,
                                    >,
                                ),
                            >(&mut store, &self.string_list)?
                            .func();
                        let record_list = *_instance
                            .get_typed_func::<
                                (&[SomeRecord],),
                                (wasmtime::component::__internal::Vec<OtherRecord>,),
                            >(&mut store, &self.record_list)?
                            .func();
                        let record_list_reverse = *_instance
                            .get_typed_func::<
                                (&[OtherRecord],),
                                (wasmtime::component::__internal::Vec<SomeRecord>,),
                            >(&mut store, &self.record_list_reverse)?
                            .func();
                        let variant_list = *_instance
                            .get_typed_func::<
                                (&[SomeVariant],),
                                (wasmtime::component::__internal::Vec<OtherVariant>,),
                            >(&mut store, &self.variant_list)?
                            .func();
                        let load_store_everything = *_instance
                            .get_typed_func::<
                                (&LoadStoreAllSizes,),
                                (LoadStoreAllSizes,),
                            >(&mut store, &self.load_store_everything)?
                            .func();
                        Ok(Guest {
                            list_u8_param,
                            list_u16_param,
                            list_u32_param,
                            list_u64_param,
                            list_s8_param,
                            list_s16_param,
                            list_s32_param,
                            list_s64_param,
                            list_float32_param,
                            list_float64_param,
                            list_u8_ret,
                            list_u16_ret,
                            list_u32_ret,
                            list_u64_ret,
                            list_s8_ret,
                            list_s16_ret,
                            list_s32_ret,
                            list_s64_ret,
                            list_float32_ret,
                            list_float64_ret,
                            tuple_list,
                            string_list_arg,
                            string_list_ret,
                            tuple_string_list,
                            string_list,
                            record_list,
                            record_list_reverse,
                            variant_list,
                            load_store_everything,
                        })
                    }
                }
                impl Guest {
                    pub fn call_list_u8_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[u8],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[u8],),
                                (),
                            >::new_unchecked(self.list_u8_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_u16_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[u16],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[u16],),
                                (),
                            >::new_unchecked(self.list_u16_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_u32_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[u32],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[u32],),
                                (),
                            >::new_unchecked(self.list_u32_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_u64_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[u64],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[u64],),
                                (),
                            >::new_unchecked(self.list_u64_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_s8_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[i8],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[i8],),
                                (),
                            >::new_unchecked(self.list_s8_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_s16_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[i16],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[i16],),
                                (),
                            >::new_unchecked(self.list_s16_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_s32_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[i32],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[i32],),
                                (),
                            >::new_unchecked(self.list_s32_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_s64_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[i64],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[i64],),
                                (),
                            >::new_unchecked(self.list_s64_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_float32_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[f32],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[f32],),
                                (),
                            >::new_unchecked(self.list_float32_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_float64_param<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[f64],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[f64],),
                                (),
                            >::new_unchecked(self.list_float64_param)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_list_u8_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<u8>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<u8>,),
                            >::new_unchecked(self.list_u8_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_u16_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<u16>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<u16>,),
                            >::new_unchecked(self.list_u16_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_u32_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<u32>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<u32>,),
                            >::new_unchecked(self.list_u32_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_u64_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<u64>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<u64>,),
                            >::new_unchecked(self.list_u64_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_s8_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<i8>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<i8>,),
                            >::new_unchecked(self.list_s8_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_s16_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<i16>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<i16>,),
                            >::new_unchecked(self.list_s16_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_s32_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<i32>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<i32>,),
                            >::new_unchecked(self.list_s32_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_s64_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<i64>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<i64>,),
                            >::new_unchecked(self.list_s64_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_float32_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<f32>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<f32>,),
                            >::new_unchecked(self.list_float32_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_list_float64_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<f64>> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (wasmtime::component::__internal::Vec<f64>,),
                            >::new_unchecked(self.list_float64_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_tuple_list<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[(u8, i8)],
                    ) -> wasmtime::Result<
                        wasmtime::component::__internal::Vec<(i64, u32)>,
                    > {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[(u8, i8)],),
                                (wasmtime::component::__internal::Vec<(i64, u32)>,),
                            >::new_unchecked(self.tuple_list)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_string_list_arg<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[wasmtime::component::__internal::String],
                    ) -> wasmtime::Result<()> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[wasmtime::component::__internal::String],),
                                (),
                            >::new_unchecked(self.string_list_arg)
                        };
                        let () = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(())
                    }
                    pub fn call_string_list_ret<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                    ) -> wasmtime::Result<
                        wasmtime::component::__internal::Vec<
                            wasmtime::component::__internal::String,
                        >,
                    > {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (),
                                (
                                    wasmtime::component::__internal::Vec<
                                        wasmtime::component::__internal::String,
                                    >,
                                ),
                            >::new_unchecked(self.string_list_ret)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), ())?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_tuple_string_list<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[(u8, wasmtime::component::__internal::String)],
                    ) -> wasmtime::Result<
                        wasmtime::component::__internal::Vec<
                            (wasmtime::component::__internal::String, u8),
                        >,
                    > {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[(u8, wasmtime::component::__internal::String)],),
                                (
                                    wasmtime::component::__internal::Vec<
                                        (wasmtime::component::__internal::String, u8),
                                    >,
                                ),
                            >::new_unchecked(self.tuple_string_list)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_string_list<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[wasmtime::component::__internal::String],
                    ) -> wasmtime::Result<
                        wasmtime::component::__internal::Vec<
                            wasmtime::component::__internal::String,
                        >,
                    > {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[wasmtime::component::__internal::String],),
                                (
                                    wasmtime::component::__internal::Vec<
                                        wasmtime::component::__internal::String,
                                    >,
                                ),
                            >::new_unchecked(self.string_list)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_record_list<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[SomeRecord],
                    ) -> wasmtime::Result<
                        wasmtime::component::__internal::Vec<OtherRecord>,
                    > {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[SomeRecord],),
                                (wasmtime::component::__internal::Vec<OtherRecord>,),
                            >::new_unchecked(self.record_list)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_record_list_reverse<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[OtherRecord],
                    ) -> wasmtime::Result<
                        wasmtime::component::__internal::Vec<SomeRecord>,
                    > {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[OtherRecord],),
                                (wasmtime::component::__internal::Vec<SomeRecord>,),
                            >::new_unchecked(self.record_list_reverse)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_variant_list<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &[SomeVariant],
                    ) -> wasmtime::Result<
                        wasmtime::component::__internal::Vec<OtherVariant>,
                    > {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&[SomeVariant],),
                                (wasmtime::component::__internal::Vec<OtherVariant>,),
                            >::new_unchecked(self.variant_list)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                    pub fn call_load_store_everything<S: wasmtime::AsContextMut>(
                        &self,
                        mut store: S,
                        arg0: &LoadStoreAllSizes,
                    ) -> wasmtime::Result<LoadStoreAllSizes> {
                        let callee = unsafe {
                            wasmtime::component::TypedFunc::<
                                (&LoadStoreAllSizes,),
                                (LoadStoreAllSizes,),
                            >::new_unchecked(self.load_store_everything)
                        };
                        let (ret0,) = callee.call(store.as_context_mut(), (arg0,))?;
                        callee.post_return(store.as_context_mut())?;
                        Ok(ret0)
                    }
                }
            }
        }
    }
}
