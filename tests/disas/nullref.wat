;;! target = "x86_64"

(module
	(func (result externref)
		(ref.null extern)
	)

	(func (result externref)
		(block (result externref)
			(ref.null extern)
		)
	)
)

;; function u0:0(i64 vmctx, i64) -> r64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @0018                               v3 = global_value.i64 gv3
;; @0018                               v4 = load.i64 notrap aligned v3+8
;; @0019                               v5 = null.r64 
;; @001b                               jump block1(v5)
;;
;;                                 block1(v2: r64):
;; @001b                               return v2
;; }
;;
;; function u0:1(i64 vmctx, i64) -> r64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @001d                               v3 = global_value.i64 gv3
;; @001d                               v4 = load.i64 notrap aligned v3+8
;; @0020                               v6 = null.r64 
;; @0022                               jump block2(v6)
;;
;;                                 block2(v5: r64):
;; @0023                               jump block1(v5)
;;
;;                                 block1(v2: r64):
;; @0023                               return v2
;; }
