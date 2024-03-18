;;! target = "x86_64"

(module
  (func (export "foo")
    i32.const 1
    ;; Fewer params than results.
    (block (param i32) (result i32 i64)
      i64.const 2
    )
    drop
    drop
  )
)

;; function u0:0(i64 vmctx, i64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @0025                               v2 = global_value.i64 gv3
;; @0025                               v3 = load.i64 notrap aligned v2+8
;; @0026                               v4 = iconst.i32 1
;; @002a                               v7 = iconst.i64 2
;; @002c                               jump block2(v4, v7)  ; v4 = 1, v7 = 2
;;
;;                                 block2(v5: i32, v6: i64):
;; @002f                               jump block1
;;
;;                                 block1:
;; @002f                               return
;; }
