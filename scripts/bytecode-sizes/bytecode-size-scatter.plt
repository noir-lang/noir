set term png size 1200,800;
set output FILEOUT;
unset key;
set title NAME;
set logscale x;
set xlabel "Base Bytecode Size (Log)";
set ylabel "Alt Bytecode Ratio";

plot FILEIN using 2:4 with points;
