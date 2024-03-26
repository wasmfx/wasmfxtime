;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

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
;;   10: auipc   a1, 0
;;   14: ld      a1, 0x48(a1)
;;   18: add     a1, a2, a1
;;   1c: bgeu    a1, a2, 8
;;   20: .byte   0x00, 0x00, 0x00, 0x00
;;   24: ld      a4, 0x58(a0)
;;   28: bltu    a4, a1, 0x2c
;;   2c: ld      a4, 0x50(a0)
;;   30: add     a2, a4, a2
;;   34: lui     a1, 0xffff
;;   38: slli    a4, a1, 4
;;   3c: add     a2, a2, a4
;;   40: sb      a3, 0(a2)
;;   44: ld      ra, 8(sp)
;;   48: ld      s0, 0(sp)
;;   4c: addi    sp, sp, 0x10
;;   50: ret
;;   54: .byte   0x00, 0x00, 0x00, 0x00
;;   58: .byte   0x01, 0x00, 0xff, 0xff
;;   5c: .byte   0x00, 0x00, 0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   60: addi    sp, sp, -0x10
;;   64: sd      ra, 8(sp)
;;   68: sd      s0, 0(sp)
;;   6c: mv      s0, sp
;;   70: auipc   a1, 0
;;   74: ld      a1, 0x48(a1)
;;   78: add     a1, a2, a1
;;   7c: bgeu    a1, a2, 8
;;   80: .byte   0x00, 0x00, 0x00, 0x00
;;   84: ld      a3, 0x58(a0)
;;   88: bltu    a3, a1, 0x2c
;;   8c: ld      a3, 0x50(a0)
;;   90: add     a2, a3, a2
;;   94: lui     a1, 0xffff
;;   98: slli    a3, a1, 4
;;   9c: add     a2, a2, a3
;;   a0: lbu     a0, 0(a2)
;;   a4: ld      ra, 8(sp)
;;   a8: ld      s0, 0(sp)
;;   ac: addi    sp, sp, 0x10
;;   b0: ret
;;   b4: .byte   0x00, 0x00, 0x00, 0x00
;;   b8: .byte   0x01, 0x00, 0xff, 0xff
;;   bc: .byte   0x00, 0x00, 0x00, 0x00
