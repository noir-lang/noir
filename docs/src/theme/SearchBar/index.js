import React from 'react';
import SearchBar from '@theme-original/SearchBar';
import AskCookbook from '@cookbookdev/docsbot/react'
import BrowserOnly from '@docusaurus/BrowserOnly';
/** It's a public API key, so it's safe to expose it here */
const COOKBOOK_PUBLIC_API_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI2NzVhMGVkODQ4MGI5OThmNzlhNjNhOTciLCJpYXQiOjE3MzM5NTUyODgsImV4cCI6MjA0OTUzMTI4OH0.NzQlH2sfhJkjvnYxoaRgSY6nRyNClxHg57n3-JueN9Q";
export default function SearchBarWrapper(props) {
  return (
    <>
      <SearchBar {...props} />
      <BrowserOnly>{() => <AskCookbook apiKey={COOKBOOK_PUBLIC_API_KEY} /> }</BrowserOnly>
    </>
  );
}
