(module
  (type $ft (func))
  (type $ct (cont $ft))

  (table $conts 0 (ref null $ct))

  (func (export "table_size") (result i32)
    (table.size $conts)
  )

  (func (export "table_grow") (result i32)
    (table.grow $conts (ref.null $ct) (i32.const 10))
  )

  (func $f)
  (elem declare func $f)

  (func (export "table_set")
    (table.set $conts (i32.const 0) (cont.new $ct (ref.func $f)))
    (table.set $conts (i32.const 1) (cont.new $ct (ref.null $ft)))
  )

  (func (export "table_get")
    (drop (table.get $conts (i32.const 0)))
  )
)

(assert_return (invoke "table_size") (i32.const 0))
(assert_return (invoke "table_grow") (i32.const 0))
(assert_return (invoke "table_grow") (i32.const 10))
(assert_return (invoke "table_set"))
(assert_return (invoke "table_get"))
