global start

section .text
bits 32
screen_width: equ 320
screen_height: equ 200
extern _start

start:
    ; Point the first entry of the level 4 page table to the first entry in the
    ; p3 table
    mov eax, p3_table
    or eax, 0b11
    mov dword [p4_table + 0], eax

    mov eax, p2_table
    or eax, 0b11
    mov dword [p3_table + 0], eax

    ; point each page table level two entry to a page
    mov ecx, 0                 ; counter variable

.map_p2_table:
    mov eax, 0x200000
    mul ecx
    or eax, 0b10000011
    mov [p2_table + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_table

    ; move page table address to cr3
    mov eax, p4_table
    mov cr3, eax

    ; enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging
    mov eax, cr0
    or eax, 1 << 31
    or eax, 1 << 16
    mov cr0, eax

	lgdt [gdt64.pointer]
    ; update selectors
    mov ax, gdt64.data
    mov ss, ax
    mov ds, ax
    mov es, ax

    jmp gdt64.code:_start

    ; ------------
    ; Page table
    ; ------------
section .bss

align 4096

p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096

section .rodata
gdt64:
        dq 0
    .code: equ $ - gdt64
        dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53)
    .data: equ $ - gdt64
        dq (1<<44) | (1<<47) | (1<<41)

    .pointer:
        dw .pointer - gdt64 - 1
        dq gdt64


        ; ------------
        ; Long mode code
        ; ------------
section .text
bits 64
long_mode_start:
    call clear_screen
    mov word [0xb8000], 0x0a48 ; H
    mov word [0xb8002], 0x0a65 ; e
    mov word [0xb8004], 0x0a6c ; l
    mov word [0xb8006], 0x0a6c ; l
    mov word [0xb8008], 0x0a6f ; o
    mov word [0xb800a], 0x0a2c ; ,
    mov word [0xb800c], 0x0a20
    mov word [0xb800e], 0x0a36
	mov word [0xb8010], 0x0a34
	mov word [0xb8012], 0x0a21 ; !
	
    hlt

clear_screen:
        enter 0,0
        push rcx
        push rbx
        push rax
        mov rax, 0xB8000       ; Text mode buffer
        mov rbx, screen_width * screen_height
        add rbx, rax
    .loop:
        cmp rax, rbx
        jge clear_screen.done

        and word [rax], 0x8F   ; clear background and flash
        or  word [rax], 0x30
        and word [rax], 0xF0   ; black text
        add rax, 2             ; move to the next text attribute byte
        jmp clear_screen.loop

    .done:
        pop rax
        pop rbx
        pop rcx
        leave
        ret

section .data
msg: db "Hello to the unknown!"