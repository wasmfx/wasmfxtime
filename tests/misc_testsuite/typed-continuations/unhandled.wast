;; Test unhandled suspension

(module
  (tag $t)

  (func $main
    (suspend $t))
)

(assert_suspension (invoke "main") "unhandled")