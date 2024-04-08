require 'pathname'
require 'json'

current_index = 0
Dir["../../test_programs/**/Nargo.toml"].each do |path_str|
  nargo_toml_path = Pathname.new(path_str)
  package_dir = nargo_toml_path.dirname
  output_json = {}
  Dir["#{package_dir}/**/*"].each do |file_path|
    file_path = Pathname.new(file_path)
    if file_path.extname.match?(/(json|toml|nr)/)
      output_json[file_path.relative_path_from(package_dir)] = File.read(file_path)
    end
  end

  json_path = "in_compile/test_package_#{current_index}.json"
  File.open(json_path, 'w') do |f|
    f.write(output_json.to_json)
  end
  current_index += 1
end

