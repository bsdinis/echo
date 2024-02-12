set terminal postscript eps colour size 24cm,8cm enhanced font 'Helvetica,20'
set output 'throughput.eps'

set border linewidth 0.75

set style data histograms
set ylabel 'Throughput B/s'
set yrange [0:*]

plot 'throughput.dat' using 2:xtic(1) title 'Throughput'
