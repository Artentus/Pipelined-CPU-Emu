package_file = open("pkg/package.json", "r")
contents = package_file.read()
package_file.close()

search_string = "\"files\": [\n"
insertion_index = contents.find(search_string) + len(search_string)
contents = contents[:insertion_index] + "    \"snippets/jam1emu-998e187e08866976/terminal.js\",\n" + contents[insertion_index:]

package_file = open("pkg/package.json", "w")
package_file.write(contents)
package_file.close()
