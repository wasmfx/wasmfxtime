(module
  (type $ft (func))
  (type $ct (cont $ft))

  ;; Tag used by generator, the i32 payload corresponds to the generated values
  (tag $yield (param i32))


  ;; Import only required for printing
  (import "wasi_snapshot_preview1" "fd_write"
     (func $wasmi_fd_write (param $fd i32)
                  (param $iovec i32)
                  (param $len i32)
                  (param $written i32) (result i32)))

  ;; Memory only required for printing
  (memory 24)
  (export "memory" (memory 0))


  ;; Printing function for unsigned integers.
  ;; This function is unrelated to stack switching.
  (func $println_u32 (param $value i32)

    ;; We use the linear memory as follows:
    ;;
    ;; Offset | Size | Content
    ;; ---------------------------------------------------------------------------
    ;;      0 |    4 | Constant i32 value:
    ;;      4 |    4 | Size of string: 2 + number of digits
    ;;      8 |   10 | Digits of $num, ASCII-encoded, with last digit at offset 17
    ;;     18 |    1 | Constant i32 value: 10 (newline)
    ;;     19 |    1 | Constant i32 value: 0 (\0)
    ;;     20 |    4 | Result area for $wasi_fd_write to put written length

    ;; index into memory where we write the next digit of $value
    (local $digit_address i32)

    (local.set $digit_address (i32.const 17))

    (loop $l
      ;; address where to write next digit
      (local.get $digit_address)
      ;; calculate next digit to print
      (i32.rem_u (local.get $value) (i32.const 10))
      ;; convert number to ASCII
      (i32.add (i32.const 48))

      (i32.store8)

      ;; decrement address
      (i32.sub (local.get $digit_address) (i32.const 1))
      (local.set $digit_address)

      ;; right-shift $value in base 10
      (i32.div_u (local.get $value) (i32.const 10))
      (local.tee $value)

      (br_if $l)
    )

    ;; Write \n and \0
    (i32.store8 (i32.const 18) (i32.const 10))
    (i32.store8 (i32.const 19) (i32.const 0))

    ;; Write iovec part1: start of string
    (i32.const 0) ;; offset
    ;; start of the string:
    (i32.add (local.get $digit_address) (i32.const 1))
    (i32.store)

    (i32.const 4)                ;; offset
    ;; value, the length of the string:
    ;; NB: 19 is address of last byte of the string
    (i32.sub (i32.const 19) (local.get $digit_address))
    (i32.store offset=0 align=2)
    (i32.const 1)   ;; 1 for stdout
    (i32.const 0)   ;; 0 as we stored the beginning of __wasi_ciovec_t
    (i32.const 1)   ;;
    (i32.const 24)  ;; nwritten
    (call $wasmi_fd_write)
    (drop)
  )


  ;; Simple generator yielding values from 100 down to 1
  (func $generator
    (local $i i32)
    (local.set $i (i32.const 100))
    (loop $l
      ;; Suspend generator, yield current value of $i to consumer
      (suspend $yield (local.get $i))
      ;; Decrement $i and exit loop once $i reaches 0
      (local.tee $i (i32.sub (local.get $i) (i32.const 1)))
      (br_if $l)
    )
  )
  (elem declare func $generator)

  (func $consumer
    (local $c (ref $ct))
    ;; Create continuation executing function $generator
    (local.set $c (cont.new $ct (ref.func $generator)))

    (loop $loop
      (block $on_yield (result i32 (ref $ct))
        ;; Resume continuation $c
        (resume $ct (on $yield $on_yield) (local.get $c))
        ;; Generator returned: no more data
        (return)
      )
      ;; Generator suspend, stack contains [i32 (ref $ct)]
      (local.set $c)
      ;; Stack now contains the i32 value yielded by generator
      (call $println_u32)

      (br $loop)
    )
  )

  (func $start (export "_start")
    (call $consumer)
  )
)
