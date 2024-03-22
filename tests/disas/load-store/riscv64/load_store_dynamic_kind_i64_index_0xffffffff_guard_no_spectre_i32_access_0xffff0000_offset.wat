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
    i32.store offset=0xffff0000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0xffff0000))

;; function u0:0:
;; block0:
;;   ld a4,88(a0)
;;   bgtu a2,a4,taken(label3),not_taken(label1)
;; block1:
;;   ld a5,80(a0)
;;   add a5,a5,a2
;;   lui a4,65535
;;   slli a0,a4,4
;;   add a5,a5,a0
;;   sw a3,0(a5)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
;;
;; function u0:1:
;; block0:
;;   ld a4,88(a0)
;;   bgtu a2,a4,taken(label3),not_taken(label1)
;; block1:
;;   ld a5,80(a0)
;;   add a5,a5,a2
;;   lui a4,65535
;;   slli a0,a4,4
;;   add a5,a5,a0
;;   lw a0,0(a5)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
