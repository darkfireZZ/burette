#!/bin/sh

sha256() {
    sha256sum $1 | cut -d ' ' -f 1
}

export HASH_DARWIN=$(sha256 $TEST_DOCS/darwin.epub)
export HASH_FAUST=$(sha256 $TEST_DOCS/faust_teil_1.epub)
export HASH_MOBY_DICK=$(sha256 $TEST_DOCS/moby_dick_1.epub)
export HASH_VAR_CHROM=$(sha256 $TEST_DOCS/var_chrom.pdf)

add_darwin() {
    burette add $TEST_DOCS/darwin.epub << EOF
On the Origin of Species By Means of Natural Selection
Y
Charles Darwin
N
N
Y
10.5962/bhl.title.59991
EOF
}
export -f add_darwin

add_faust() {
    burette add $TEST_DOCS/faust_teil_1.epub << EOF
Faust: Eine Tragödie [erster Teil]
Yes
Johann Wolfgang von Goethe
No
No
No
EOF
}
export -f add_faust

add_moby_dick() {
    burette add $TEST_DOCS/moby_dick_1.epub << EOF
Moby Dick; Or, The Whale
yes
Herman Melville
no
yes
978-0198853695
yes
9788417517212
no
no
EOF
}
export -f add_moby_dick

add_var_chrom() {
    burette add $TEST_DOCS/var_chrom.pdf << EOF
Variations Chromatiques de concert
YES
Georges Bizet
NO
NO
NO
EOF
}
export -f add_var_chrom
