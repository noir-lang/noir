require 'fileutils'
require 'pathname'

# for each file in tmp:
# - make test in compile_success_empty
# - use file name as crate name
# - make crate w/ Nargo.toml
# - put contents of file in main.nr
Dir['tmp/*'].each do |tmp_path|
  original_tmp_path = tmp_path
  tmp_path = Pathname.new tmp_path
  tmp_path = tmp_path.basename.to_s

  # tests_dir = Pathname.new("../../test_programs/compile_success_empty")
  tests_dir = Pathname.new("../../test_programs/compile_success_no_bug")
  test_dir = tests_dir.join(tmp_path)
  nargo_path = test_dir.join('Nargo.toml')
  src_dir = test_dir.join('src')
  main_path = src_dir.join('main.nr')
  FileUtils.mkdir_p src_dir

  p tmp_path
  File.write(nargo_path, "[package]\nname = \"#{tmp_path}\"\ntype = \"bin\"\nauthors = [\"\"]\n\n[dependencies]\n")
  File.write(main_path, File.read(original_tmp_path))
end

