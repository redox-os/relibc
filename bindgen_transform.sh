sed -i 's/::std::os::raw:://g' $1
perl -i -p0e 's/extern "C" \{\n    pub fn/#[no_mangle]\npub extern "C" fn/g' $1 
perl -i -p0e 's/;\n\}/ {\n    unimplemented!();\n\}\n/g' $1
rustfmt $1
