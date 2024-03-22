;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

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

;; function u0:0:
;; block0:
;;   slli a2,a2,32
;;   srli a5,a2,32
;;   ld a4,[const(0)]
;;   sltu a4,a4,a5
;;   ld a0,80(a0)
;;   add a5,a0,a5
;;   lui a0,1
;;   add a5,a5,a0
;;   sub a1,zero,a4
;;   not a4,a1
;;   and a5,a5,a4
;;   sb a3,0(a5)
;;   j label1
;; block1:
;;   ret
;;
;; function u0:1:
;; block0:
;;   slli a2,a2,32
;;   srli a4,a2,32
;;   ld a3,[const(0)]
;;   sltu a3,a3,a4
;;   ld a5,80(a0)
;;   add a4,a5,a4
;;   lui a5,1
;;   add a4,a4,a5
;;   sub a1,zero,a3
;;   not a3,a1
;;   and a5,a4,a3
;;   lbu a0,0(a5)
;;   j label1
;; block1:
;;   ret
