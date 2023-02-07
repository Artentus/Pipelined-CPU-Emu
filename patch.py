from pathlib import Path
import os

package_file = open("pkg/package.json", "r")
contents = package_file.read()
package_file.close()

search_string = "\"files\": [\n"
insertion_index = contents.find(search_string) + len(search_string)

js_files = list(Path("./pkg/snippets").rglob("*.js"))
modified_contents = contents[:insertion_index]
for js_file in js_files:
    modified_contents += "    \"" + str(os.path.relpath(js_file, "./pkg")).replace("\\", "/") + "\",\n"
modified_contents += contents[insertion_index:]

package_file = open("pkg/package.json", "w")
package_file.write(modified_contents)
package_file.close()
