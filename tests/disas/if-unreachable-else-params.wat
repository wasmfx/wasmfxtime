;;! target = "x86_64"

(module
  (type (;0;) (func (param i32)))
  (func $main (type 0) (param i32)
    i32.const 35
    loop (param i32)  ;; label = @1
      local.get 0
      if (param i32)  ;; label = @2
        i64.load16_s align=1
        unreachable
        unreachable
        unreachable
        unreachable
        unreachable
        local.get 0
        unreachable
        unreachable
        i64.load8_u offset=11789
        unreachable
      else
        i32.popcnt
        local.set 0
        return
        unreachable
      end
      unreachable
      unreachable
      nop
      f32.lt
      i32.store8 offset=82
      unreachable
    end
    unreachable
    unreachable
    unreachable
    unreachable)
  (table (;0;) 63 255 funcref)
  (memory (;0;) 13 16)
  (export "t1" (table 0))
  (export "m1" (memory 0))
  (export "main" (func $main))
  (export "memory" (memory 0)))

;; function u0:0(i64 vmctx, i64, i32) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned readonly checked gv3+96
;;     sig0 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;;                                     v7 -> v2
;; @0042                               v3 = global_value.i64 gv3
;; @0042                               v4 = load.i64 notrap aligned v3+8
;; @0043                               v5 = iconst.i32 35
;; @0045                               jump block2(v5)  ; v5 = 35
;;
;;                                 block2(v6: i32):
;; @0049                               brif.i32 v7, block4, block6(v6)
;;
;;                                 block4:
;; @004b                               v9 = uextend.i64 v6
;; @004b                               v10 = global_value.i64 gv4
;; @004b                               v11 = iadd v10, v9
;; @004b                               v12 = sload16.i64 little heap v11
;; @004e                               trap unreachable
;;
;;                                 block6(v8: i32):
;; @005d                               v13 = popcnt.i32 v6
;; @0060                               return
;; }
