;;! target = "x86_64"
;;! flags = ["has_popcnt", "has_sse42"]

(module
    (func (param i64) (result i64)
      local.get 0
      i64.popcnt
    )
)
;;      	 55                   	push	rbp
;;      	 4889e5               	mov	rbp, rsp
;;      	 4883ec10             	sub	rsp, 0x10
;;      	 4d8b5e08             	mov	r11, qword ptr [r14 + 8]
;;      	 4d8b1b               	mov	r11, qword ptr [r11]
;;      	 4939e3               	cmp	r11, rsp
;;      	 0f8719000000         	ja	0x31
;;   18:	 48897c2408           	mov	qword ptr [rsp + 8], rdi
;;      	 4c893424             	mov	qword ptr [rsp], r14
;;      	 488b442408           	mov	rax, qword ptr [rsp + 8]
;;      	 f3480fb8c0           	popcnt	rax, rax
;;      	 4883c410             	add	rsp, 0x10
;;      	 5d                   	pop	rbp
;;      	 c3                   	ret	
;;   31:	 0f0b                 	ud2	
