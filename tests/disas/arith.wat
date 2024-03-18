;;! target = "x86_64"

(module
  (memory 1)
  (func $main (local i32)
      (local.set 0 (i32.sub (i32.const 4) (i32.const 4)))
      (if
          (local.get 0)
          (then unreachable)
          (else (drop (i32.mul (i32.const 6) (local.get 0))))
       )
  )
  (start $main)
  (data (i32.const 0) "abcdefgh")
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
;; @001f                               v2 = iconst.i32 0
;; @001f                               v3 = global_value.i64 gv3
;; @001f                               v4 = load.i64 notrap aligned v3+8
;; @0021                               v5 = iconst.i32 4
;; @0023                               v6 = iconst.i32 4
;; @0025                               v7 = isub v5, v6  ; v5 = 4, v6 = 4
;; @002a                               brif v7, block2, block4
;;
;;                                 block2:
;; @002c                               trap unreachable
;;
;;                                 block4:
;; @002e                               v8 = iconst.i32 6
;; @0032                               v9 = imul v8, v7  ; v8 = 6
;; @0034                               jump block3
;;
;;                                 block3:
;; @0035                               jump block1
;;
;;                                 block1:
;; @0035                               return
;; }
