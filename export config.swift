#!/usr/bin/swift

import Foundation

func camelCaseToSnakeCase(_ string: Substring) -> String {
	var output = string.first?.lowercased() ?? ""
	
	var previousCharacterWasUppercase = false
	
	for character in string.dropFirst() {
		if character.isUppercase {
			if previousCharacterWasUppercase {
				output.append(character.lowercased())
			} else {
				output += "_\(character.lowercased())"
				previousCharacterWasUppercase = true
			}
		} else if character.isNumber {
			output += "_\(character)"
			
			previousCharacterWasUppercase = false
		} else {
			output.append(character)
			
			previousCharacterWasUppercase = false
		}
	}
	
	return output
}

let cargoTOMLPath = URL(filePath: "Cargo.toml")
let cargoTOML = try String(contentsOf: cargoTOMLPath, encoding: .utf8)

let versionNumber = cargoTOML.matches(of: #/version = "(?'number'[\d\.]+)"/#).first!.output.number

let defaultConfigPath = URL(filePath: "src/config/default.rs")

let lines = try String(contentsOf: defaultConfigPath, encoding: .utf8)
	.split(separator: "\n", omittingEmptySubsequences: false)
	.dropFirst(14)
	.dropLast(6)

precondition(lines.first!.contains("Mode::Normal"))

var output = """
	{%- highlight toml -%}
	#:schema https://simonomi.dev/hexapoda/config/schema-v\(versionNumber).json
	
	
	"""
var mode: String?

for line in lines {
	if let match = line.wholeMatch(of: #/.*Mode::(?'mode'\w*),.*/#) {
		mode = match.output.mode.lowercased()
	} else if line.contains("None") {
		output += "[\(mode!)]\n"
	} else if let match = line.wholeMatch(of: #/.*PartialAction::(?'partialAction'\w*).*/#) {
		let partialAction = match.output.partialAction.lowercased()
		output += "[\(mode!).\(partialAction)]\n"
	} else if let match = line.wholeMatch(of: #/.*\(keypress\("(?'keypress'.*?)"\), (?'action'.*?)\.into\(\)\).*/#) {
		if match.output.keypress.contains(where: { !($0.isLetter || $0.isNumber) }) {
			output += "\"\(match.output.keypress)\" = \"\(camelCaseToSnakeCase(match.output.action))\"\n"
		} else {
			output += "\(match.output.keypress) = \"\(camelCaseToSnakeCase(match.output.action))\"\n"
		}
	} else {
		output += "\n"
	}
}
output += "{%- endhighlight -%}\n"

let outputPath = URL(filePath: "~/Documents/programming/websites/simonomi.dev/_includes/hexapoda/hexapoda v\(versionNumber).toml")
try Data(output.utf8).write(to: outputPath)

print("wrote config to \(outputPath.path(percentEncoded: false))")
