;;! target = "x86_64"
;;! test = "clif"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store offset=0)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0))

;; function u0:0(i64 vmctx, i64, i64, i32) tail {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1+16
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+104
;;     gv5 = load.i64 notrap aligned checked gv3+96
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i64, v3: i32):
;; @0040                               v4 = load.i64 notrap aligned v0+104
;; @0040                               v5 = iconst.i64 4
;; @0040                               v6 = isub v4, v5  ; v5 = 4
;; @0040                               v7 = icmp ugt v2, v6
;; @0040                               v8 = load.i64 notrap aligned checked v0+96
;; @0040                               v9 = iadd v8, v2
;; @0040                               v10 = iconst.i64 0
;; @0040                               v11 = select_spectre_guard v7, v10, v9  ; v10 = 0
;; @0040                               store little heap v3, v11
;; @0043                               jump block1
;;
;;                                 block1:
;; @0043                               return
;; }
;;
;; function u0:1(i64 vmctx, i64, i64) -> i32 tail {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1+16
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+104
;;     gv5 = load.i64 notrap aligned checked gv3+96
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i64):
;; @0048                               v4 = load.i64 notrap aligned v0+104
;; @0048                               v5 = iconst.i64 4
;; @0048                               v6 = isub v4, v5  ; v5 = 4
;; @0048                               v7 = icmp ugt v2, v6
;; @0048                               v8 = load.i64 notrap aligned checked v0+96
;; @0048                               v9 = iadd v8, v2
;; @0048                               v10 = iconst.i64 0
;; @0048                               v11 = select_spectre_guard v7, v10, v9  ; v10 = 0
;; @0048                               v12 = load.i32 little heap v11
;; @004b                               jump block1
;;
;;                                 block1:
;; @004b                               return v12
;; }
