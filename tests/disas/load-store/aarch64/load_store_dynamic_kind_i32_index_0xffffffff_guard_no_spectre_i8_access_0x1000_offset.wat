;;! target = "aarch64"
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
    i32.store8 offset=0x1000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load8_u offset=0x1000))

;; wasm[0]::function[0]:
;;    0: stp     x29, x30, [sp, #-0x10]!
;;    4: mov     x29, sp
;;    8: ldr     x8, [x0, #0x58]
;;    c: mov     w9, w2
;;   10: cmp     x9, x8
;;   14: b.hi    #0x2c
;;   18: ldr     x10, [x0, #0x50]
;;   1c: add     x10, x10, #1, lsl #12
;;   20: strb    w3, [x10, w2, uxtw]
;;   24: ldp     x29, x30, [sp], #0x10
;;   28: ret
;;   2c: .byte   0x1f, 0xc1, 0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   40: stp     x29, x30, [sp, #-0x10]!
;;   44: mov     x29, sp
;;   48: ldr     x8, [x0, #0x58]
;;   4c: mov     w9, w2
;;   50: cmp     x9, x8
;;   54: b.hi    #0x6c
;;   58: ldr     x10, [x0, #0x50]
;;   5c: add     x9, x10, #1, lsl #12
;;   60: ldrb    w0, [x9, w2, uxtw]
;;   64: ldp     x29, x30, [sp], #0x10
;;   68: ret
;;   6c: .byte   0x1f, 0xc1, 0x00, 0x00
