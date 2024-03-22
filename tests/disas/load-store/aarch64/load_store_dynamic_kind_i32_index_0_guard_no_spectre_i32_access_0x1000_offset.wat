;;! target = "aarch64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store offset=0x1000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load offset=0x1000))

;; function u0:0:
;; block0:
;;   ldr x10, [x0, #88]
;;   mov w11, w2
;;   movz x12, #4100
;;   sub x10, x10, x12
;;   subs xzr, x11, x10
;;   b.hi label3 ; b label1
;; block1:
;;   ldr x12, [x0, #80]
;;   add x12, x12, #4096
;;   str w3, [x12, w2, UXTW]
;;   b label2
;; block2:
;;   ret
;; block3:
;;   udf #0xc11f
;;
;; function u0:1:
;; block0:
;;   ldr x10, [x0, #88]
;;   mov w11, w2
;;   movz x12, #4100
;;   sub x10, x10, x12
;;   subs xzr, x11, x10
;;   b.hi label3 ; b label1
;; block1:
;;   ldr x12, [x0, #80]
;;   add x11, x12, #4096
;;   ldr w0, [x11, w2, UXTW]
;;   b label2
;; block2:
;;   ret
;; block3:
;;   udf #0xc11f
