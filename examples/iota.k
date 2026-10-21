// run: iota 1 32
// expect: param 0 = 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
kernel iota(out: *u32) {
    out[thread_idx.x] = thread_idx.x;
}
