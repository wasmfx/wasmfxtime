;;! target = "x86_64"

(module
  (func (export "param") (param i32) (result i32)
    (i32.const 1)
    (if (param i32) (result i32) (local.get 0)
      (then (i32.const 2) (i32.add))
      (else (i32.const -2) (i32.add))
    )
  )
)

;; function u0:0(i64 vmctx, i64, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @0023                               v4 = global_value.i64 gv3
;; @0023                               v5 = load.i64 notrap aligned v4+8
;; @0024                               v6 = iconst.i32 1
;; @0028                               brif v2, block2, block4(v6)  ; v6 = 1
;;
;;                                 block2:
;; @002a                               v8 = iconst.i32 2
;; @002c                               v9 = iadd.i32 v6, v8  ; v6 = 1, v8 = 2
;; @002d                               jump block3(v9)
;;
;;                                 block4(v10: i32):
;; @002e                               v11 = iconst.i32 0xffff_fffe
;; @0030                               v12 = iadd.i32 v6, v11  ; v6 = 1, v11 = 0xffff_fffe
;; @0031                               jump block3(v12)
;;
;;                                 block3(v7: i32):
;; @0032                               jump block1(v7)
;;
;;                                 block1(v3: i32):
;; @0032                               return v3
;; }
