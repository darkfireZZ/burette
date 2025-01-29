#!/bin/sh

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

