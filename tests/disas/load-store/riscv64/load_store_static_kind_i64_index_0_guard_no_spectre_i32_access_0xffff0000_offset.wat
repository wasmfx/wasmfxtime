;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -W memory64 -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

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
;;   lui a4,16
;;   addi a1,a4,-4
;;   bgtu a2,a1,taken(label3),not_taken(label1)
;; block1:
;;   ld a0,80(a0)
;;   add a0,a0,a2
;;   lui a5,65535
;;   slli a1,a5,4
;;   add a0,a0,a1
;;   sw a3,0(a0)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
;;
;; function u0:1:
;; block0:
;;   lui a4,16
;;   addi a1,a4,-4
;;   bgtu a2,a1,taken(label3),not_taken(label1)
;; block1:
;;   ld a0,80(a0)
;;   add a0,a0,a2
;;   lui a5,65535
;;   slli a1,a5,4
;;   add a0,a0,a1
;;   lw a0,0(a0)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
