;;; talon-mode.el --- Major mode for TALON exploit development language

;; Copyright (C) 2026 TALON Project
;; Author: TALON Project
;; Keywords: languages, security, exploit
;; Version: 0.1.0

;;; Commentary:
;; Major mode for editing TALON exploit development scripts.
;; Provides syntax highlighting, indentation, and basic editing support.

;;; Code:

(defvar talon-mode-hook nil
  "Hook run when entering TALON mode.")

(defvar talon-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map "\C-j" 'newline-and-indent)
    map)
  "Keymap for TALON major mode.")

;; Syntax highlighting
(defconst talon-font-lock-keywords
  (list
   ;; Control flow keywords
   '("\\<\\(if\\|else\\|end\\|for\\|in\\|while\\|try\\|catch\\|match\\|when\\|default\\|break\\|continue\\|return\\)\\>" . font-lock-keyword-face)
   
   ;; Declaration keywords
   '("\\<\\(let\\|const\\|def\\|macro\\|import\\|from\\|as\\)\\>" . font-lock-keyword-face)
   
   ;; Exploit keywords
   '("\\<\\(analyze\\|connect\\|send\\|recv\\|print\\|shellcode\\|payload\\|exploit\\|rop\\|heap\\|kernel\\)\\>" . font-lock-keyword-face)
   
   ;; Advanced exploit keywords
   '("\\<\\(auto_rop\\|heap_exploit\\|auto_exploit\\|taint_analysis\\|symbolic_exec\\|binary_similarity\\|notebook\\|export_to\\|note\\)\\>" . font-lock-builtin-face)
   
   ;; Safety keywords
   '("\\<\\(enable_strict_mode\\|disable_safety\\|set_timeout\\|set_max_memory\\|set_recursion_depth\\)\\>" . font-lock-warning-face)
   
   ;; Types
   '("\\<\\(int\\|str\\|bytes\\|list\\|map\\|set\\|bool\\|null\\)\\>" . font-lock-type-face)
   
   ;; Constants
   '("\\<\\(true\\|false\\|null\\)\\>" . font-lock-constant-face)
   
   ;; Builtin functions
   '("\\<\\(p64\\|p32\\|p16\\|p8\\|u64\\|u32\\|u16\\|u8\\|len\\|bytes\\|hex\\|decode\\|encode\\)\\>" . font-lock-function-name-face)
   
   ;; Exploit functions
   '("\\<\\(leak_pie_base\\|leak_canary\\|leak_libc\\|find_gadgets\\|build_rop_chain\\)\\>" . font-lock-function-name-face)
   
   ;; Libc functions
   '("\\<\\(malloc\\|free\\|mprotect\\|mmap\\|execve\\|system\\)\\>" . font-lock-function-name-face)
   
   ;; Network functions
   '("\\<\\(connect_to\\|send_payload\\|receive_data\\|interactive\\)\\>" . font-lock-function-name-face)
   
   ;; Exploit techniques
   '("\\<\\(buffer-overflow\\|format-string\\|ret2libc\\|ret2csu\\|ret2plt\\|fsop\\|srop\\|house-of-force\\|off-by-one\\|use-after-free\\)\\>" . font-lock-preprocessor-face)
   
   '("\\<\\(tcache_poisoning\\|fastbin_attack\\|unsorted_bin_attack\\|one_gadget\\|ret2syscall\\|mprotect_rwx\\|stack_pivot\\)\\>" . font-lock-preprocessor-face)
   
   ;; Numbers
   '("\\<0x[0-9a-fA-F]+\\>" . font-lock-constant-face)
   '("\\<0b[01]+\\>" . font-lock-constant-face)
   '("\\<[0-9]+\\>" . font-lock-constant-face)
   
   ;; Operators
   '("\\(\\.\\.\\.|\\.\\.\\||>\\|[+\\-*/%=<>!&|^~]\\)" . font-lock-operator-face)
   
   ;; Special keywords
   '("\\<\\(to\\|on\\|port\\|technique\\|target\\|objective\\|constraints\\|strategy\\|glibc_version\\|overwrite_with\\)\\>" . font-lock-constant-face))
  "Keyword highlighting specification for `talon-mode'.")

;; Syntax table
(defvar talon-mode-syntax-table
  (let ((st (make-syntax-table)))
    ;; C-style comments
    (modify-syntax-entry ?/ ". 124b" st)
    (modify-syntax-entry ?* ". 23" st)
    (modify-syntax-entry ?\n "> b" st)
    ;; Strings
    (modify-syntax-entry ?\" "\"" st)
    (modify-syntax-entry ?\' "\"" st)
    ;; Underscores are part of words
    (modify-syntax-entry ?_ "w" st)
    st)
  "Syntax table for `talon-mode'.")

;; Indentation
(defun talon-indent-line ()
  "Indent current line as TALON code."
  (interactive)
  (beginning-of-line)
  (if (bobp)
      (indent-line-to 0)
    (let ((not-indented t) cur-indent)
      (if (looking-at "^[ \t]*end\\>")
          (progn
            (save-excursion
              (forward-line -1)
              (setq cur-indent (- (current-indentation) 2)))
            (if (< cur-indent 0)
                (setq cur-indent 0)))
        (save-excursion
          (while not-indented
            (forward-line -1)
            (if (looking-at "^[ \t]*end\\>")
                (progn
                  (setq cur-indent (current-indentation))
                  (setq not-indented nil))
              (if (looking-at "^[ \t]*\\(def\\|if\\|for\\|while\\|try\\|match\\|auto_rop\\|heap_exploit\\|notebook\\)\\>")
                  (progn
                    (setq cur-indent (+ (current-indentation) 2))
                    (setq not-indented nil))
                (if (bobp)
                    (setq not-indented nil)))))))
      (if cur-indent
          (indent-line-to cur-indent)
        (indent-line-to 0)))))

;; Mode definition
;;;###autoload
(define-derived-mode talon-mode prog-mode "TALON"
  "Major mode for editing TALON exploit development scripts."
  :syntax-table talon-mode-syntax-table
  (setq-local comment-start "// ")
  (setq-local comment-end "")
  (setq-local font-lock-defaults '(talon-font-lock-keywords))
  (setq-local indent-line-function 'talon-indent-line)
  (setq-local tab-width 2)
  (setq-local indent-tabs-mode nil))

;; Associate .tal and .talon files with talon-mode
;;;###autoload
(add-to-list 'auto-mode-alist '("\\.tal\\'" . talon-mode))
;;;###autoload
(add-to-list 'auto-mode-alist '("\\.talon\\'" . talon-mode))

(provide 'talon-mode)
;;; talon-mode.el ends here
