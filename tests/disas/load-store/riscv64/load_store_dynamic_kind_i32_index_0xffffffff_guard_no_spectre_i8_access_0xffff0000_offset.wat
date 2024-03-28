;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -O static-memory-maximum-size=0 -O static-memory-guard-size=4294967295 -O dynamic-memory-guard-size=4294967295"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store8 offset=0xffff0000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load8_u offset=0xffff0000))

;; wasm[0]::function[0]:
;;       addi    sp, sp, -0x10
;;       sd      ra, 8(sp)
;;       sd      s0, 0(sp)
;;       mv      s0, sp
;;       ld      a1, 0x58(a0)
;;       slli    a5, a2, 0x20
;;       srli    a2, a5, 0x20
;;       bltu    a1, a2, 0x2c
;;       ld      a1, 0x50(a0)
;;       add     a1, a1, a2
;;       lui     a0, 0xffff
;;       slli    a2, a0, 4
;;       add     a1, a1, a2
;;       sb      a3, 0(a1)
;;       ld      ra, 8(sp)
;;       ld      s0, 0(sp)
;;       addi    sp, sp, 0x10
;;       ret
;;       .byte   0x00, 0x00, 0x00, 0x00
;;
;; wasm[0]::function[1]:
;;       addi    sp, sp, -0x10
;;       sd      ra, 8(sp)
;;       sd      s0, 0(sp)
;;       mv      s0, sp
;;       ld      a1, 0x58(a0)
;;       slli    a5, a2, 0x20
;;       srli    a2, a5, 0x20
;;       bltu    a1, a2, 0x2c
;;       ld      a1, 0x50(a0)
;;       add     a1, a1, a2
;;       lui     a0, 0xffff
;;       slli    a2, a0, 4
;;       add     a1, a1, a2
;;       lbu     a0, 0(a1)
;;       ld      ra, 8(sp)
;;       ld      s0, 0(sp)
;;       addi    sp, sp, 0x10
;;       ret
;;       .byte   0x00, 0x00, 0x00, 0x00
