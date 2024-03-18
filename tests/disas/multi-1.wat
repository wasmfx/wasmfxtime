;;! target = "x86_64"

(module
  (func (export "multiBlock") (param i64 i32) (result i32 i64 f64)
    (local.get 1)
    (local.get 0)
    (block (param i32 i64) (result i32 i64 f64)
      (f64.const 1234.5))))

;; function u0:0(i64 vmctx, i64, i64, i32) -> i32, i64, f64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i64, v3: i32):
;; @0033                               v7 = global_value.i64 gv3
;; @0033                               v8 = load.i64 notrap aligned v7+8
;; @003a                               v12 = f64const 0x1.34a0000000000p10
;; @0043                               jump block2(v3, v2, v12)  ; v12 = 0x1.34a0000000000p10
;;
;;                                 block2(v9: i32, v10: i64, v11: f64):
;; @0044                               jump block1(v9, v10, v11)
;;
;;                                 block1(v4: i32, v5: i64, v6: f64):
;; @0044                               return v4, v5, v6
;; }
