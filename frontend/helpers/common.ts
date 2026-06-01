export const postFormatDateTime = (unixtime: number): string => {
    if (!unixtime) return '';
    const date = new Date(unixtime * 1000);

    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');

    return `${day}.${month}.${year} ${hours}:${minutes}`;
};

export function removeEmptyStrings<T extends Record<string, any>>(obj: T): Partial<T> {
    const result: Partial<T> = {};
    
    for (const key in obj) {
        if (!!obj[key] && obj[key] !== '') {
            result[key] = obj[key];
        }
    }
    
    return result;
}