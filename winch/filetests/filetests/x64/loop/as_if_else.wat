;;! target = "x86_64"
(module
  (func (export "as-if-else") (result i32)
    (if (result i32) (i32.const 1) (then (i32.const 2)) (else (loop (result i32) (i32.const 1))))
  )
)
;;      	 55                   	push	rbp
;;      	 4889e5               	mov	rbp, rsp
;;      	 4d8b5e08             	mov	r11, qword ptr [r14 + 8]
;;      	 4d8b1b               	mov	r11, qword ptr [r11]
;;      	 4981c308000000       	add	r11, 8
;;      	 4939e3               	cmp	r11, rsp
;;      	 0f872a000000         	ja	0x45
;;   1b:	 4883ec08             	sub	rsp, 8
;;      	 4c893424             	mov	qword ptr [rsp], r14
;;      	 b801000000           	mov	eax, 1
;;      	 85c0                 	test	eax, eax
;;      	 0f840a000000         	je	0x3a
;;   30:	 b802000000           	mov	eax, 2
;;      	 e905000000           	jmp	0x3f
;;   3a:	 b801000000           	mov	eax, 1
;;      	 4883c408             	add	rsp, 8
;;      	 5d                   	pop	rbp
;;      	 c3                   	ret	
;;   45:	 0f0b                 	ud2	
