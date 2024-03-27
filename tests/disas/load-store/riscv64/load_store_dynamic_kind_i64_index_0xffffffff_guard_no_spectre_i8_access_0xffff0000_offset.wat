;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=4294967295 -O dynamic-memory-guard-size=4294967295"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store8 offset=0xffff0000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load8_u offset=0xffff0000))

;; wasm[0]::function[0]:
;;    0: addi    sp, sp, -0x10
;;    4: sd      ra, 8(sp)
;;    8: sd      s0, 0(sp)
;;    c: mv      s0, sp
;;   10: ld      a4, 0x58(a0)
;;   14: bltu    a4, a2, 0x2c
;;   18: ld      a5, 0x50(a0)
;;   1c: add     a5, a5, a2
;;   20: lui     a4, 0xffff
;;   24: slli    a0, a4, 4
;;   28: add     a5, a5, a0
;;   2c: sb      a3, 0(a5)
;;   30: ld      ra, 8(sp)
;;   34: ld      s0, 0(sp)
;;   38: addi    sp, sp, 0x10
;;   3c: ret
;;   40: .byte   0x00, 0x00, 0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   44: addi    sp, sp, -0x10
;;   48: sd      ra, 8(sp)
;;   4c: sd      s0, 0(sp)
;;   50: mv      s0, sp
;;   54: ld      a4, 0x58(a0)
;;   58: bltu    a4, a2, 0x2c
;;   5c: ld      a5, 0x50(a0)
;;   60: add     a5, a5, a2
;;   64: lui     a4, 0xffff
;;   68: slli    a0, a4, 4
;;   6c: add     a5, a5, a0
;;   70: lbu     a0, 0(a5)
;;   74: ld      ra, 8(sp)
;;   78: ld      s0, 0(sp)
;;   7c: addi    sp, sp, 0x10
;;   80: ret
;;   84: .byte   0x00, 0x00, 0x00, 0x00
