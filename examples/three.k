// run: three 3
// expect: param 0 = 1
// expect: param 1 = 2
// expect: param 2 = 3
kernel three(a: *u32, b: *u32, c: *u32) {
    *a = 1;
    *b = 2;
    *c = 3;
}
