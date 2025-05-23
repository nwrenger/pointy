export const keyToDisplayMap: Record<string, string> = {
	' ': 'Space',
	Minus: '-',
	Equal: '=',
	Plus: '+',
	Comma: ',',
	Period: '.',
	IntlBackslash: '§',
	Slash: '/',
	Backslash: '\\',
	Enter: '↵',
	ArrowUp: '↑',
	ArrowDown: '↓',
	ArrowLeft: '←',
	ArrowRight: '→',
	Escape: 'Esc',
	Command: '⌘',
	Meta: '⌘',
	Control: 'Ctrl',
	Alt: '⌥',
	Shift: '⇧'
};

export const keyToShortcutMap: Record<string, string> = {
	Meta: 'Command',
	IntlBackslash: '§',
	' ': 'Space'
};

export function keyCodeToKey(keyCode: string): string {
	if (keyCode.startsWith('Key')) {
		return keyCode.slice(3);
	}
	if (keyCode.endsWith('Left')) {
		return keyCode.slice(0, -4);
	}
	if (keyCode.startsWith('Digit')) {
		return keyCode.slice(5);
	}
	if (keyCode.endsWith('Right')) {
		return keyCode.slice(0, -5);
	}
	return keyCode;
}

export function keyToDisplay(keyCode: string): string {
	const mappedChar = keyToDisplayMap[keyCodeToKey(keyCode)];
	if (mappedChar) {
		return mappedChar;
	} else {
		return keyCode;
	}
}

export function keyToShortcut(key: string): string {
	const mappedChar = keyToShortcutMap[key];
	if (mappedChar) {
		return mappedChar;
	} else {
		return key;
	}
}

export function keyCombToDisplay(keyComb: string[]): string {
	return keyComb.map(keyToDisplay).join('+');
}

export function keyCombToShortcut(keyComb: string[]): string {
	return keyComb.map(keyToShortcut).join('+');
}

export function isShortcut(letters: string[]): boolean {
	// letters contain at least one modifier key and one non-modifier key

	let hasModifier = false;
	let hasNonModifier = false;

	for (const letter of letters) {
		if (modifierKeySet.has(letter)) {
			hasModifier = true;
		} else {
			hasNonModifier = true;
		}
	}

	return hasModifier && hasNonModifier;
}

export const modifierKeySet = new Set(['Meta', 'Shift', 'Alt', 'Control']);
