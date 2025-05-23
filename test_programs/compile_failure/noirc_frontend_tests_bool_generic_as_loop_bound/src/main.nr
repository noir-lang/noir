
    pub fn read<let N: bool>() {
        let mut fields = [0; N];
        for i in 0..N { 
            fields[i] = i + 1;
        }
        assert(fields[0] == 1);
    }
    