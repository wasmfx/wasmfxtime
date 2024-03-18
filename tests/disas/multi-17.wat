;;! target = "x86_64"

(module
  (func $main (type 0) (param i32 i32 i32) (result i32)
    i32.const 0
    i32.const 0
    i32.const 0
    i32.const 0

    i32.const 0
    if (param i32 i32 i32) (result i32)  ;; label = @1
      br 0 (;@1;)
    else
      call $main
    end

    i32.const 0

    i32.const 0
    if (param i32 i32 i32) (result i32)  ;; label = @1
      drop
      drop
    else
      drop
      drop
    end
  )
  (export "main" (func $main)))

;; function u0:0(i64 vmctx, i64, i32, i32, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i64, i32, i32, i32) -> i32 fast
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     fn0 = colocated u0:0 sig0
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: i32, v4: i32):
;; @0024                               v6 = global_value.i64 gv3
;; @0024                               v7 = load.i64 notrap aligned v6+8
;; @0025                               v8 = iconst.i32 0
;; @0027                               v9 = iconst.i32 0
;; @0029                               v10 = iconst.i32 0
;; @002b                               v11 = iconst.i32 0
;; @002d                               v12 = iconst.i32 0
;; @002f                               brif v12, block2, block4(v9, v10, v11)  ; v12 = 0, v9 = 0, v10 = 0, v11 = 0
;;
;;                                 block2:
;; @0031                               jump block3(v11)  ; v11 = 0
;;
;;                                 block4(v14: i32, v15: i32, v16: i32):
;; @0034                               v17 = call fn0(v0, v0, v9, v10, v11)  ; v9 = 0, v10 = 0, v11 = 0
;; @0036                               jump block3(v17)
;;
;;                                 block3(v13: i32):
;; @0037                               v18 = iconst.i32 0
;; @0039                               v19 = iconst.i32 0
;; @003b                               brif v19, block5, block7(v8, v13, v18)  ; v19 = 0, v8 = 0, v18 = 0
;;
;;                                 block5:
;; @003f                               jump block6(v8)  ; v8 = 0
;;
;;                                 block7(v21: i32, v22: i32, v23: i32):
;; @0042                               jump block6(v8)  ; v8 = 0
;;
;;                                 block6(v20: i32):
;; @0043                               jump block1(v20)
;;
;;                                 block1(v5: i32):
;; @0043                               return v5
;; }
