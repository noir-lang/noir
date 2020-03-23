
// #define SQR(a)                                                                                                          \
//         "movq 0(" a "), %%rdx                      \n\t" /* load a[0] into %rdx */                                      \
//                                                                                                                         \
//         "xorq %%r8, %%r8                           \n\t" /* clear flags                                             */  \
//         /* compute a[0] *a[1], a[0]*a[2], a[0]*a[3], a[1]*a[2], a[1]*a[3], a[2]*a[3]                                */  \
//         "mulxq %%rdx, %%r8, %%r12 \n\t" \
//         "mulxq 8(" a "), %%r9, %%r13               \n\t" /* (r[1], r[2]) <- a[0] * a[1]                             */  \
//         "mulxq 16(" a "), %%r10, %%r14              \n\t" /* (t[1], t[2]) <- a[0] * a[2]                             */  \
//         "mulxq 24(" a "), %%r11, %%r15             \n\t" /* (r[3], r[4]) <- a[0] * a[3]                             */  \
//         "adcxq %%r9, %%r9 \n\t" \
//         "adoxq %%r12, %%r9 \n\t" \
//         "adcxq %%r13, %%r10 \n\t" \
//         "adoxq %%r10, %%r10 \n\t" \
//         "adcxq %%r14, %%r11 \n\t" \
//         "adoxq %%r11, %%r11 \n\t" \
//         "adcxq %[zero_reference], %%r15 \n\t" \
//         "adoxq %%r15, %%r15 \n\t" \
//         /* perform modular reduction: r[0] */                                                                           \
//         "movq %%r8, %%rdx                          \n\t" /* move r8 into %rdx                                       */  \
//         "mulxq %[r_inv], %%rdx, %%rdi              \n\t" /* (%rdx, _) <- k = r[9] * r_inv                           */  \
//         "mulxq %[modulus_0], %%rdi, %%rcx          \n\t" /* (t[0], t[1]) <- (modulus[0] * k)                        */  \
//         "adcxq %%rdi, %%r8                         \n\t" /* r[0] += t[0] (%r8 now free)                             */  \
//         "adoxq %%rcx, %%r9                         \n\t" /* r[1] += t[1] + flag_c                                   */  \
//         "mulxq %[modulus_1], %%rdi, %%rcx          \n\t" /* (t[2], t[3]) <- (modulus[1] * k)                        */  \
//         "adcxq %%rdi, %%r9                         \n\t" /* r[1] += t[2]                                            */  \
//         "adoxq %%rcx, %%r10                        \n\t" /* r[2] += t[3] + flag_o                                   */  \
//         "mulxq %[modulus_2], %%rdi, %%rcx          \n\t" /* (t[0], t[1]) <- (modulus[3] * k)                        */  \
//         "adcxq %%rdi, %%r10                        \n\t" /* r[2] += t[0] + flag_c                                   */  \
//         "adoxq %%rcx, %%r11                        \n\t" /* r[3] += t[1] + flag_o                                   */  \
//         "mulxq %[modulus_3], %%rdi, %%rcx           \n\t" /* (t[2], t[3]) <- (modulus[2] * k)                        */  \
//         "adcxq %%rdi, %%r11                         \n\t" /* r[3] += t[2] + flag_c                                   */  \
//         "adoxq %%rcx, %%r15                        \n\t" /* t[4] += t[3] + flag_o                                   */  \
//         "adcxq %[zero_reference], %%r15            \n\t" /* t[4] += flag_c                                          */  \
//                                                                                                                         \
//         "movq %%r15, %%r12 \n\t" \
//         "movq 8(" a "), %%rdx                      \n\t" /* load a[0] into %rdx */                                      \
//         "mulxq %%rdx, %%r8, %%r15 \n\t" \
//         "mulxq 16(" a "), %%r13, %%r14              \n\t" /* (t[1], t[2]) <- a[0] * a[2]                             */  \
//         "mulxq 24(" a "), %%rdi, %%rdx             \n\t" /* (r[3], r[4]) <- a[0] * a[3]                             */  \
//         "adcxq %%r13, %%r13 \n\t" \
//         "adoxq %%r15, %%r13 \n\t" \
//         "adcxq %%r14, %%rdi \n\t" \
//         "adoxq %%rdi, %%rdi \n\t" \
//         "adcxq %[zero_reference], %%rdx \n\t" \
//         "adoxq %%rdx, %%rdx \n\t" \
//         \
//         "adcxq %%r8, %%r10 \n\t" \
//         "adcxq %%r13, %%r11 \n\t" \
//         "adcxq %%rdi, %%r12 \n\t" \
//         "adcxq %[zero_reference], %%rdx \n\t" \
//         "movq %%rdx, %%r15 \n\t" \
//                                                                                                                         \
//         "movq %%r9, %%rdx                          \n\t" /* move r8 into %rdx                                       */  \
//         "mulxq %[r_inv], %%rdx, %%rdi              \n\t" /* (%rdx, _) <- k = r[9] * r_inv                           */  \
//         "mulxq %[modulus_0], %%rdi, %%rcx          \n\t" /* (t[0], t[1]) <- (modulus[0] * k)                        */  \
//         "adcxq %%rdi, %%r9                         \n\t" /* r[0] += t[0] (%r8 now free)                             */  \
//         "adoxq %%rcx, %%r10                         \n\t" /* r[1] += t[1] + flag_c                                   */  \
//         "mulxq %[modulus_1], %%rdi, %%rcx          \n\t" /* (t[2], t[3]) <- (modulus[1] * k)                        */  \
//         "adcxq %%rdi, %%r10                         \n\t" /* r[1] += t[2]                                            */  \
//         "adoxq %%rcx, %%r11                        \n\t" /* r[2] += t[3] + flag_o                                   */  \
//         "mulxq %[modulus_2], %%rdi, %%rcx          \n\t" /* (t[0], t[1]) <- (modulus[3] * k)                        */  \
//         "adcxq %%rdi, %%r11                        \n\t" /* r[2] += t[0] + flag_c                                   */  \
//         "adoxq %%rcx, %%r12                        \n\t" /* r[3] += t[1] + flag_o                                   */  \
//         "mulxq %[modulus_3], %%rdi, %%rcx           \n\t" /* (t[2], t[3]) <- (modulus[2] * k)                        */  \
//         "adcxq %%rdi, %%r12                         \n\t" /* r[3] += t[2] + flag_c                                   */  \
//         "adoxq %%rcx, %%r15                        \n\t" /* t[4] += t[3] + flag_o                                   */  \
//         "adcxq %[zero_reference], %%r15            \n\t" /* t[4] += flag_c                                          */  \
//         "movq %%r15, %%r13 \n\t" \
//         \
//         "movq 16(" a "), %%rdx \n\t" \
//         "mulxq %%rdx, %%r8, %%r9 \n\t" \
//         "mulxq 24(" a "), %%r14, %%r15 \n\t" \
//         "adcxq %%r14, %%r14 \n\t" \
//         "adoxq %%r9, %%r14 \n\t" \
//         "adcxq %%r15, %%r15 \n\t" \
//         "adoxq %[zero_reference], %%r15 \n\t" \
//         "adcxq %%r8, %%r12 \n\t" \
//         "adcxq %%r14, %%r13 \n\t" \
//         "adcxq %[zero_reference], %%r15 \n\t" \
//         "movq %%r10, %%rdx                          \n\t" /* move r8 into %rdx                                       */  \
//         "mulxq %[r_inv], %%rdx, %%rdi              \n\t" /* (%rdx, _) <- k = r[9] * r_inv                           */  \
//         "mulxq %[modulus_0], %%rdi, %%rcx          \n\t" /* (t[0], t[1]) <- (modulus[0] * k)                        */  \
//         "adcxq %%rdi, %%r10                         \n\t" /* r[0] += t[0] (%r8 now free)                             */  \
//         "adoxq %%rcx, %%r11                         \n\t" /* r[1] += t[1] + flag_c                                   */  \
//         "mulxq %[modulus_1], %%rdi, %%rcx          \n\t" /* (t[2], t[3]) <- (modulus[1] * k)                        */  \
//         "adcxq %%rdi, %%r11                         \n\t" /* r[1] += t[2]                                            */  \
//         "adoxq %%rcx, %%r12                        \n\t" /* r[2] += t[3] + flag_o                                   */  \
//         "mulxq %[modulus_2], %%rdi, %%rcx          \n\t" /* (t[0], t[1]) <- (modulus[3] * k)                        */  \
//         "adcxq %%rdi, %%r12                        \n\t" /* r[2] += t[0] + flag_c                                   */  \
//         "adoxq %%rcx, %%r13                        \n\t" /* r[3] += t[1] + flag_o                                   */  \
//         "mulxq %[modulus_3], %%rdi, %%rcx           \n\t" /* (t[2], t[3]) <- (modulus[2] * k)                        */  \
//         "adcxq %%rdi, %%r13                         \n\t" /* r[3] += t[2] + flag_c                                   */  \
//         "adoxq %%rcx, %%r15                        \n\t" /* t[4] += t[3] + flag_o                                   */  \
//         "adcxq %[zero_reference], %%r15            \n\t" /* t[4] += flag_c                                          */  \
//         "movq %%r15, %%r14 \n\t" \
//         \
//         "movq 24(" a "), %%rdx \n\t " \
//         "mulxq %%rdx, %%r8, %%r15 \n\t" \
//         "adcxq %%r8, %%r14 \n\t" \
//         "adcxq %[zero_reference], %%r15 \n\t" \
//         "movq %%r11, %%rdx                          \n\t" /* move r8 into %rdx                                       */  \
//         "mulxq %[r_inv], %%rdx, %%rdi              \n\t" /* (%rdx, _) <- k = r[9] * r_inv                           */  \
//         "mulxq %[modulus_0], %%rdi, %%rcx          \n\t" /* (t[0], t[1]) <- (modulus[0] * k)                        */  \
//         "adcxq %%rdi, %%r11                         \n\t" /* r[0] += t[0] (%r8 now free)                             */  \
//         "adoxq %%rcx, %%r12                         \n\t" /* r[1] += t[1] + flag_c                                   */  \
//         "mulxq %[modulus_1], %%rdi, %%rcx          \n\t" /* (t[2], t[3]) <- (modulus[1] * k)                        */  \
//         "adcxq %%rdi, %%r12                         \n\t" /* r[1] += t[2]                                            */  \
//         "adoxq %%rcx, %%r13                        \n\t" /* r[2] += t[3] + flag_o                                   */  \
//         "mulxq %[modulus_2], %%rdi, %%rcx          \n\t" /* (t[0], t[1]) <- (modulus[3] * k)                        */  \
//         "adcxq %%rdi, %%r13                        \n\t" /* r[2] += t[0] + flag_c                                   */  \
//         "adoxq %%rcx, %%r14                        \n\t" /* r[3] += t[1] + flag_o                                   */  \
//         "mulxq %[modulus_3], %%rdi, %%rcx           \n\t" /* (t[2], t[3]) <- (modulus[2] * k)                        */  \
//         "adcxq %%rdi, %%r14                         \n\t" /* r[3] += t[2] + flag_c                                   */  \
//         "adoxq %%rcx, %%r15                        \n\t" /* t[4] += t[3] + flag_o                                   */  \
//         "adcxq %[zero_reference], %%r15            \n\t" /* t[4] += flag_c                                          */
