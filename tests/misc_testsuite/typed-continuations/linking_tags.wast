(module $foo
  (tag $foo (export "foo"))
)
(register "foo")

(module $bar
  (type $ft (func))
  (type $ct (cont $ft))
  (tag $foo (import "foo" "foo"))
  (tag $bar)
  (func $do_foo
    (suspend $foo))

  (func $main (export "main")
    (block $on_bar (result (ref $ct))
      (resume $ct (tag $bar $on_bar) (cont.new $ct (ref.func $do_foo)))
      (unreachable)
    )
    (unreachable))
  (elem declare func $do_foo)
)
(register "bar")
(assert_suspension (invoke "main") "unhandled")