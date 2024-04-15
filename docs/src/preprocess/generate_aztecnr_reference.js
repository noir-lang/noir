const fs = require('fs');
const path = require('path');

function listNrFiles(dir, fileList = []) {
    const files = fs.readdirSync(dir);
    files.forEach(file => {
        const filePath = path.join(dir, file);
        const stat = fs.statSync(filePath);
        if (stat.isDirectory()) {
            listNrFiles(filePath, fileList);
        } else if (filePath.endsWith('.nr') && !file.endsWith('lib.nr')) {
            fileList.push(filePath);
        }
    });
    return fileList;
}

function escapeHtml(unsafeText) {
    if (!unsafeText) {
        // Return an empty string or some default value if unsafeText is undefined or null
        return '';
    }
    return unsafeText.replace(/</g, "&lt;").replace(/>/g, "&gt;");
}


function parseParameters(paramString) {
    if (!paramString.trim()) {
        return [];
    }

    return paramString.split(',').map(param => {
        param = param.trim().replace(/[\[:;,.]$/g, '').replace(/^[\[:;,.]/g, ''); // Clean up start and end
        let [paramName, type] = param.split(':').map(p => p.trim());
        return { name: paramName, type: escapeHtml(type) };
    });
}


function parseStruct(content) {
    const structRegex = /struct (\w+)\s*{([\s\S]*?)}/g;
    let match;
    const structs = [];

    while ((match = structRegex.exec(content)) !== null) {
        const structName = match[1];
        const fields = match[2].trim().split('\n').map(fieldLine => {
            fieldLine = fieldLine.trim().replace(/,$/, '');
            // Skip lines that are comments or do not contain a colon (indicating they are not field definitions)
            if (!fieldLine.startsWith('//') && fieldLine.includes(':')) {
                let [name, type] = fieldLine.split(/:\s*/);
                return { name, type };
            }
        }).filter(field => field !== undefined); // Filter out undefined entries resulting from comments or invalid lines

        let descriptionLines = [];
        let lineIndex = content.lastIndexOf('\n', match.index - 1);
        while (lineIndex >= 0) {
            let endOfPreviousLine = content.lastIndexOf('\n', lineIndex - 1);
            let line = content.substring(endOfPreviousLine + 1, lineIndex).trim();

            if (line.startsWith('//') && !line.includes('docs:start:') && !line.includes('docs:end:')) {
                descriptionLines.unshift(line.replace('//', '').trim());
            } else if (!line.startsWith('//')) {
                break;
            }

            lineIndex = endOfPreviousLine;
        }

        let description = descriptionLines.join(' ');
        structs.push({ structName, fields, description });
    }

    return structs;
}

function parseFunctions(content) {
    const functions = [];
    const implRegex = /impl\s+(\w+)\s*{/g;
    let implMatch;

    while ((implMatch = implRegex.exec(content)) !== null) {
        const structName = implMatch[1];
        let braceDepth = 1;
        let currentPos = implMatch.index + implMatch[0].length;

        while (braceDepth > 0 && currentPos < content.length) {
            if (content[currentPos] === '{') {
                braceDepth++;
            } else if (content[currentPos] === '}') {
                braceDepth--;
            }
            currentPos++;
        }

        const implBlockContent = content.substring(implMatch.index, currentPos);
        const methodRegex =
            /(?:pub\s+)?fn\s+(\w+)(?:<.*?>)?\s*\(([\s\S]*?)\)\s*(?:->\s*(.*?))?\s*{/g;
        let methodMatch;

        while ((methodMatch = methodRegex.exec(implBlockContent)) !== null) {
            const name = methodMatch[1];
            const params = parseParameters(methodMatch[2]);
            const returnType = (methodMatch[3] || '').replace(/[\[:;,.]$/g, '').replace(/^[\[:;,.]/g, '');

            let description = '';
            let commentIndex = methodMatch.index;
            while (commentIndex >= 0) {
                const commentMatch = implBlockContent.substring(0, commentIndex).match(/\/\/\s*(.*)\n\s*$/);
                if (commentMatch && !commentMatch[1].includes('docs:start:') && !commentMatch[1].includes('docs:end:')) {
                    description = commentMatch[1] + (description ? ' ' + description : '');
                    commentIndex = commentMatch.index - 1;
                } else {
                    break;
                }
            }

            functions.push({ structName, name, params, returnType, description, isMethod: true });
        }
    }

    const standaloneFunctionRegex = /(?:pub\s+)?fn\s+(\w+)(?:<.*?>)?\s*\(([\s\S]*?)\)\s*(?:->\s*(.*?))?\s*{/g;
    let standaloneFunctionMatch;
    while ((standaloneFunctionMatch = standaloneFunctionRegex.exec(content)) !== null) {
        const name = standaloneFunctionMatch[1];

        if (!functions.some(f => f.name === name && f.isMethod)) {
            const params = parseParameters(standaloneFunctionMatch[2]);
            const returnType = (standaloneFunctionMatch[3] || '').replace(/[\[:;,.]$/g, '').replace(/^[\[:;,.]/g, '');

            let description = '';
            const descriptionMatch = content.substring(0, standaloneFunctionMatch.index).match(/\/\/\s*(.*)\n\s*$/);
            if (descriptionMatch) {
                const precedingText = content.substring(0, descriptionMatch.index);
                if (!precedingText.includes('docs:start:') && !precedingText.includes('docs:end:')) {
                    description = descriptionMatch[1];
                }
            }

            functions.push({ name, params, returnType, description, isMethod: false });
        }
    }

    return functions;
}

function generateMarkdown(structs, functions) {
    let markdown = '';

    structs.forEach(structInfo => {
        if (structInfo) {
            markdown += `# ${escapeHtml(structInfo.structName)}\n\n`;

            if (structInfo.description) {
                markdown += `${escapeHtml(structInfo.description)}\n\n`;
            }

            if (structInfo.fields.length > 0) {
                markdown += `## Fields\n`;
                markdown += `| Field | Type |\n| --- | --- |\n`;
                structInfo.fields.forEach(field => {
                    const cleanType = escapeHtml(field.type.replace(/[\[:;,]$/g, '').replace(/^[\[:;,]/g, ''));
                    const fieldName = escapeHtml(field.name.replace(/[:;]/g, ''));
                    markdown += `| ${fieldName} | ${cleanType} |\n`;
                });
                markdown += '\n';
            }

            // Filter methods for this struct
            const methods = functions.filter(f => f.isMethod && f.structName === escapeHtml(structInfo.structName));
            if (methods.length > 0) {
                markdown += `## Methods\n\n`;
                methods.forEach(func => {
                    markdown += `### ${escapeHtml(func.name)}\n\n`;

                   
                    // Description taken from a comment above the function decalaration
                    // If the comment is docs:, looks at the comment above
                    if (func.description) {
                        markdown += `${escapeHtml(func.description)}\n\n`;
                    }

                     // Codeblock for example usage
                     const usageParams = func.params.map(param => param.name).join(', ');
                     markdown += "```rust\n" + `${func.structName}::${func.name}(${usageParams});` + "\n```\n\n";

                    // Parameters
                    if (func.params.length > 0) {
                        markdown += `#### Parameters\n`;
                        markdown += `| Name | Type |\n| --- | --- |\n`;
                        func.params.forEach(({ name, type }) => {
                            markdown += `| ${escapeHtml(name)} | ${escapeHtml(type)} |\n`;
                        });
                        markdown += '\n';
                    } else {
                        markdown += 'Takes no parameters.\n\n';
                    }

                    // Returns
                    if (func.returnType) {
                        markdown += `#### Returns\n`;
                        markdown += `| Type |\n| --- |\n`;
                        markdown += `| ${escapeHtml(func.returnType)} |\n\n`;
                    }
                });
            }
        }
    });

    // Generate markdown for standalone functions
    const standaloneFunctions = functions.filter(f => !f.isMethod);
    if (standaloneFunctions.length > 0) {
        markdown += `## Standalone Functions\n\n`;
        standaloneFunctions.forEach(func => {
            markdown += `### ${escapeHtml(func.name)}\n\n`;

            // Insert usage code block
            const usageParams = func.params.map(param => param.name).join(', ');
            markdown += "```rust\n" + `${func.name}(${usageParams});` + "\n```\n\n";

            if (func.description) {
                markdown += `${escapeHtml(func.description)}\n\n`;
            }

            if (func.params.length > 0) {
                markdown += `#### Parameters\n`;
                markdown += `| Name | Type |\n| --- | --- |\n`;
                func.params.forEach(({ name, type }) => {
                    markdown += `| ${escapeHtml(name)} | ${escapeHtml(type)} |\n`;
                });
                markdown += '\n';
            } else {
                markdown += 'Takes no parameters.\n\n';
            }

            if (func.returnType) {
                markdown += `#### Returns\n`;
                markdown += `| Type |\n| --- |\n`;
                markdown += `| ${escapeHtml(func.returnType)} |\n\n`;
            }
        });
    }

    return markdown;
}


function processFiles(baseDir, outputBaseDir) {
    const nrFiles = listNrFiles(baseDir);
    let docStructure = {}; // To hold structured documentation paths

    nrFiles.forEach(filePath => {
        
        const content = fs.readFileSync(filePath, 'utf8');
        const structs = parseStruct(content);
        const functions = parseFunctions(content);

        if (structs.length === 0 && functions.length === 0) {
            return;
        }

        const markdown = generateMarkdown(structs, functions);

        const relativePath = path.relative(baseDir, filePath);
        const adjustedPath = relativePath.replace('/src', '').replace(/\.nr$/, '.md');
        const outputFilePath = path.join(outputBaseDir, adjustedPath);

        fs.mkdirSync(path.dirname(outputFilePath), { recursive: true });
        fs.writeFileSync(outputFilePath, markdown);

        // Adjusted to populate docStructure for JSON
        const docPathForJson = adjustedPath.replace(/\\/g, '/').replace('.md', '');
        const parts = docPathForJson.split('/');
        let current = docStructure;

        for (let i = 0; i < parts.length - 1; i++) {
            current[parts[i]] = current[parts[i]] || {};
            current = current[parts[i]];
        }

        current._docs = current._docs || [];
        current._docs.push(parts[parts.length - 1]);
    });

    // Write structured documentation paths to JSON
    const outputPath = path.join(__dirname, 'AztecnrReferenceAutogenStructure.json');
    fs.writeFileSync(outputPath, JSON.stringify({ AztecNR: docStructure }, null, 2));
}


let baseDir = path.resolve(__dirname, '../../../noir-projects/aztec-nr');
let outputBaseDir = path.resolve(__dirname, '../../docs/developers/contracts/references/aztec-nr');

// if (process.env.CI === 'true') {
//     baseDir = path.resolve(__dirname, '../noir-projects/aztec-nr');
//     outputBaseDir = path.resolve(__dirname, '../../docs/developers/contracts/references/aztec-nr');
// }

processFiles(baseDir, outputBaseDir);

