import { useCallback, useState } from 'react';
import _ from 'lodash';

type useGetFileTextHookType = () => [
  (file: File) => void,
  string,
  boolean,
  Error | undefined,
];

const useGetFileText: useGetFileTextHookType = () => {
  const [fileText, setFileText] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<Error | undefined>(undefined);

  const handleFileLoad = useCallback((event: ProgressEvent<FileReader>) => {
    const result = event.target?.result;
    if (_.isString(result)) {
      setFileText(result);
    } else {
      setError(
        new Error(
          `Failed to parse file contents to string. Got: ${JSON.stringify(result)}`,
        ),
      );
    }
    setIsLoading(false);
  }, []);

  const processFile = useCallback(
    (file: File) => {
      setIsLoading(true);
      const reader = new FileReader();
      reader.onload = handleFileLoad;
      reader.readAsText(file);
    },
    [handleFileLoad],
  );

  return [processFile, fileText, isLoading, error];
};

export { useGetFileText };
