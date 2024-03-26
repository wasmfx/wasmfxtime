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
    i32.store offset=0x1000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0x1000))

;; wasm[0]::function[0]:
;;    0: addi    sp, sp, -0x10
;;    4: sd      ra, 8(sp)
;;    8: sd      s0, 0(sp)
;;    c: mv      s0, sp
;;   10: ld      a4, 0x58(a0)
;;   14: lui     a5, 1
;;   18: addi    a5, a5, 4
;;   1c: sub     a4, a4, a5
;;   20: bltu    a4, a2, 0x28
;;   24: ld      a5, 0x50(a0)
;;   28: add     a5, a5, a2
;;   2c: lui     t6, 1
;;   30: add     t6, t6, a5
;;   34: sw      a3, 0(t6)
;;   38: ld      ra, 8(sp)
;;   3c: ld      s0, 0(sp)
;;   40: addi    sp, sp, 0x10
;;   44: ret
;;   48: .byte   0x00, 0x00, 0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   4c: addi    sp, sp, -0x10
;;   50: sd      ra, 8(sp)
;;   54: sd      s0, 0(sp)
;;   58: mv      s0, sp
;;   5c: ld      a4, 0x58(a0)
;;   60: lui     a3, 1
;;   64: addi    a5, a3, 4
;;   68: sub     a4, a4, a5
;;   6c: bltu    a4, a2, 0x28
;;   70: ld      a5, 0x50(a0)
;;   74: add     a5, a5, a2
;;   78: lui     t6, 1
;;   7c: add     t6, t6, a5
;;   80: lw      a0, 0(t6)
;;   84: ld      ra, 8(sp)
;;   88: ld      s0, 0(sp)
;;   8c: addi    sp, sp, 0x10
;;   90: ret
;;   94: .byte   0x00, 0x00, 0x00, 0x00
