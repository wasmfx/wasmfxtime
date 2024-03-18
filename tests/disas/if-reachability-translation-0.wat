;;! target = "x86_64"

;; An unreachable `if` means that the consequent, alternative, and following
;; block are also unreachable.

(module
  (func (param i32) (result i32)
    unreachable
    if  ;; label = @2
      nop
    else
      nop
    end
    i32.const 0))

;; function u0:0(i64 vmctx, i64, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @0018                               v4 = global_value.i64 gv3
;; @0018                               v5 = load.i64 notrap aligned v4+8
;; @0019                               trap unreachable
;; }
