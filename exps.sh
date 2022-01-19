cargo run --release -- exp1 -n 10 -l 10000 -a 1.1 -o exp1_s1.csv -p 1 --step-sigma 1 --num-sigma 50 
cargo run --release -- exp1 -n 10 -l 10000 -a 1.1 -o exp1_s5.csv -p 1 --step-sigma 5 --num-sigma 100 
cargo run --release -- exp1 -n 10 -l 10000 -a 1.1 -o exp1_s25.csv -p 1 --step-sigma 25 --num-sigma 100 


cargo run --release -- exp2 -n 10 -l 10000 -a 1.1 -t 10 -s 10.0 -o exp2_s10.csv   
cargo run --release -- exp2 -n 10 -l 10000 -a 1.1 -t 10 -s 100.0 -o exp2_s100.csv     
cargo run --release -- exp2 -n 10 -l 10000 -a 1.1 -t 10 -s 1000.0 -o exp2_s1000.csv   

cargo run --release -- exp2 -n 10 -l 10000 -a 1.1 -t 10 -s 1.0 -o exp2_rs1.csv   --rel-sigma
cargo run --release -- exp2 -n 10 -l 10000 -a 1.1 -t 10 -s 10.0 -o exp2_rs10.csv   --rel-sigma
cargo run --release -- exp2 -n 10 -l 10000 -a 1.1 -t 10 -s 100.0 -o exp2_rs100.csv   --rel-sigma
