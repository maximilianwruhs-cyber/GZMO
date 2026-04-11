#!/usr/bin/env bash
# Autonomously write a new capability script into the agent's skillset. Usage: ./skills/create_skill.sh <filename.sh>

if [ -z "$1" ]; then
    echo "Usage: $0 <filename.sh>"
    echo "Please provide the file name. The script will open a standard input stream or you can use shell_exec with echo."
    exit 1
fi

FILE="./skills/$1"

if [ -f "$FILE" ]; then
    echo "Error: Skill $FILE already exists. Use shell_exec with sed or echo to modify it."
    exit 1
fi

echo "Creating new skill: $FILE"
echo "You must now execute a \`shell_exec\` tool call using bash to write the logic into $FILE:"
echo "Example:"
echo '{"command": "cat << '"'EOF'"' > '"$FILE"'\n#!/usr/bin/env bash\n# description of what this does\necho \"hello world\"\nEOF\nchmod +x '"$FILE"'"}'
