;;! target = "x86_64"

(module
  (func (export "as-loop-first") (result i32)
    (block (result i32) (loop (result i32) (br 1 (i32.const 3)) (i32.const 2)))
  )
)
;;      	 55                   	push	rbp
;;      	 4889e5               	mov	rbp, rsp
;;      	 4d8b5e08             	mov	r11, qword ptr [r14 + 8]
;;      	 4d8b1b               	mov	r11, qword ptr [r11]
;;      	 4981c308000000       	add	r11, 8
;;      	 4939e3               	cmp	r11, rsp
;;      	 0f8713000000         	ja	0x2e
;;   1b:	 4883ec08             	sub	rsp, 8
;;      	 4c893424             	mov	qword ptr [rsp], r14
;;      	 b803000000           	mov	eax, 3
;;      	 4883c408             	add	rsp, 8
;;      	 5d                   	pop	rbp
;;      	 c3                   	ret	
;;   2e:	 0f0b                 	ud2	
