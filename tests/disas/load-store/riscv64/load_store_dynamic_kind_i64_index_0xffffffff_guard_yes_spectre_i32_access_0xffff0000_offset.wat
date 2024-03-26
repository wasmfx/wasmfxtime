;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=4294967295 -O dynamic-memory-guard-size=4294967295"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store offset=0xffff0000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0xffff0000))

;; wasm[0]::function[0]:
;;    0: addi    sp, sp, -0x10
;;    4: sd      ra, 8(sp)
;;    8: sd      s0, 0(sp)
;;    c: mv      s0, sp
;;   10: ld      a4, 0x58(a0)
;;   14: ld      a5, 0x50(a0)
;;   18: sltu    a4, a4, a2
;;   1c: add     a2, a5, a2
;;   20: lui     a1, 0xffff
;;   24: slli    a5, a1, 4
;;   28: add     a2, a2, a5
;;   2c: neg     a0, a4
;;   30: not     a4, a0
;;   34: and     a4, a2, a4
;;   38: sw      a3, 0(a4)
;;   3c: ld      ra, 8(sp)
;;   40: ld      s0, 0(sp)
;;   44: addi    sp, sp, 0x10
;;   48: ret
;;
;; wasm[0]::function[1]:
;;   4c: addi    sp, sp, -0x10
;;   50: sd      ra, 8(sp)
;;   54: sd      s0, 0(sp)
;;   58: mv      s0, sp
;;   5c: ld      a3, 0x58(a0)
;;   60: ld      a4, 0x50(a0)
;;   64: sltu    a3, a3, a2
;;   68: add     a2, a4, a2
;;   6c: lui     a1, 0xffff
;;   70: slli    a4, a1, 4
;;   74: add     a2, a2, a4
;;   78: neg     a0, a3
;;   7c: not     a3, a0
;;   80: and     a4, a2, a3
;;   84: lw      a0, 0(a4)
;;   88: ld      ra, 8(sp)
;;   8c: ld      s0, 0(sp)
;;   90: addi    sp, sp, 0x10
;;   94: ret
