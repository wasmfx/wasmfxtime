;;! target = "aarch64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

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
;;    0: stp     x29, x30, [sp, #-0x10]!
;;    4: mov     x29, sp
;;    8: mov     w14, w2
;;    c: mov     w15, #-0xffff
;;   10: adds    x14, x14, x15
;;   14: b.hs    #0x48
;;   18: ldr     x15, [x0, #0x58]
;;   1c: ldr     x1, [x0, #0x50]
;;   20: mov     x0, #0
;;   24: add     x1, x1, w2, uxtw
;;   28: mov     x2, #0xffff0000
;;   2c: add     x1, x1, x2
;;   30: cmp     x14, x15
;;   34: csel    x0, x0, x1, hi
;;   38: csdb
;;   3c: strb    w3, [x0]
;;   40: ldp     x29, x30, [sp], #0x10
;;   44: ret
;;   48: .byte   0x1f, 0xc1, 0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   60: stp     x29, x30, [sp, #-0x10]!
;;   64: mov     x29, sp
;;   68: mov     w14, w2
;;   6c: mov     w15, #-0xffff
;;   70: adds    x14, x14, x15
;;   74: b.hs    #0xa8
;;   78: ldr     x15, [x0, #0x58]
;;   7c: ldr     x1, [x0, #0x50]
;;   80: mov     x0, #0
;;   84: add     x1, x1, w2, uxtw
;;   88: mov     x2, #0xffff0000
;;   8c: add     x1, x1, x2
;;   90: cmp     x14, x15
;;   94: csel    x0, x0, x1, hi
;;   98: csdb
;;   9c: ldrb    w0, [x0]
;;   a0: ldp     x29, x30, [sp], #0x10
;;   a4: ret
;;   a8: .byte   0x1f, 0xc1, 0x00, 0x00
