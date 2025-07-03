import os

def check_large_files(directory, min_lines=500):
    """
    检查指定目录中行数超过min_lines的文件
    :param directory: 要检查的目录路径
    :param min_lines: 最小行数阈值
    :return: 包含大文件信息的列表
    """
    large_files = []
    
    for root, _, files in os.walk(directory):
        for file in files:
            if not file.endswith('.rs'):  # 只检查Rust文件
                continue
                
            file_path = os.path.join(root, file)
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    line_count = sum(1 for _ in f)
                    if line_count > min_lines:
                        large_files.append({
                            'path': file_path,
                            'lines': line_count,
                            'relative_path': os.path.relpath(file_path, directory)
                        })
            except Exception as e:
                print(f"Error reading {file_path}: {e}")
    
    return large_files

if __name__ == '__main__':
    src_dir = os.path.join(os.path.dirname(__file__), '..', 'src')
    large_files = check_large_files(src_dir)
    
    print(f"Found {len(large_files)} files with more than 500 lines:")
    for file in sorted(large_files, key=lambda x: x['lines'], reverse=True):
        print(f"{file['relative_path']}: {file['lines']} lines")