;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

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
;;   slli a4,a2,32
;;   srli a1,a4,32
;;   lui a4,262144
;;   addi a5,a4,-1025
;;   slli a2,a5,2
;;   bgtu a1,a2,taken(label3),not_taken(label1)
;; block1:
;;   ld a0,80(a0)
;;   add a0,a0,a1
;;   sw a3,4096(a0)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
;;
;; function u0:1:
;; block0:
;;   slli a4,a2,32
;;   srli a1,a4,32
;;   lui a3,262144
;;   addi a5,a3,-1025
;;   slli a2,a5,2
;;   bgtu a1,a2,taken(label3),not_taken(label1)
;; block1:
;;   ld a0,80(a0)
;;   add a0,a0,a1
;;   lw a0,4096(a0)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
