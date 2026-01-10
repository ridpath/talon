" Vim syntax file for TALON exploit development language
" Language: TALON
" Maintainer: TALON Project
" Latest Revision: 2026-01-08

if exists("b:current_syntax")
  finish
endif

" Keywords
syn keyword talonKeyword analyze connect send recv let const def macro if else end for in while try catch return import from as match when default break continue print shellcode payload exploit rop heap kernel
syn keyword talonKeyword auto_rop heap_exploit auto_exploit taint_analysis symbolic_exec binary_similarity
syn keyword talonKeyword enable_strict_mode disable_safety set_timeout set_max_memory set_recursion_depth
syn keyword talonKeyword notebook export_to note

" Control flow
syn keyword talonControl if else end for in while try catch match when default break continue return

" Builtin functions
syn keyword talonBuiltin p64 p32 p16 p8 u64 u32 u16 u8 len bytes hex decode encode
syn keyword talonBuiltin leak_pie_base leak_canary leak_libc find_gadgets build_rop_chain
syn keyword talonBuiltin malloc free mprotect mmap execve system
syn keyword talonBuiltin connect_to send_payload receive_data interactive

" Types
syn keyword talonType int str bytes list map set null bool

" Constants
syn keyword talonConstant true false null

" Numbers
syn match talonNumber '\<\d\+\>'
syn match talonNumber '\<0x[0-9a-fA-F]\+\>'
syn match talonNumber '\<0b[01]\+\>'

" Strings
syn region talonString start='"' end='"' skip='\\"' contains=talonEscape
syn region talonString start="'" end="'" skip="\\'" contains=talonEscape
syn match talonEscape '\\[nrt\\"]' contained

" Comments
syn match talonComment '//.*$'
syn region talonComment start='/\*' end='\*/'

" Operators
syn match talonOperator '[+\-*/%=<>!&|^~]'
syn match talonOperator '\.\.\.'
syn match talonOperator '|>'

" Special commands
syn keyword talonSpecial connect to on port recv send print analyze binary
syn keyword talonSpecial technique target objective constraints strategy
syn keyword talonSpecial glibc_version overwrite_with

" Exploit-specific keywords
syn keyword talonExploit buffer-overflow rop format-string heap kernel ret2libc ret2csu ret2plt
syn keyword talonExploit fsop srop house-of-force off-by-one use-after-free
syn keyword talonExploit tcache_poisoning fastbin_attack unsorted_bin_attack
syn keyword talonExploit one_gadget ret2syscall mprotect_rwx stack_pivot

" Function definitions
syn region talonFunction start='def\s\+\w\+' end='end' fold contains=ALL

" Highlighting
hi def link talonKeyword Keyword
hi def link talonControl Conditional
hi def link talonBuiltin Function
hi def link talonType Type
hi def link talonConstant Constant
hi def link talonNumber Number
hi def link talonString String
hi def link talonEscape SpecialChar
hi def link talonComment Comment
hi def link talonOperator Operator
hi def link talonSpecial Special
hi def link talonExploit PreProc
hi def link talonFunction Function

let b:current_syntax = "talon"
