set term png size 1200,800;
set output FILEOUT;
unset key;
set title NAME;
set logscale x;
set xlabel "Base Bytecode Size (Log)";
set ylabel "Alt Bytecode Ratio";

plot FILEIN using X_COL:RATIO_COL with points;
