// Given a local benchmark json aggregated file, reformats it in markdown
// and comments on the PR that prompted it. If the CI is rerun, the comment
// is updated.
import { createConsoleLogger } from '@aztec/foundation/log';

import * as https from 'https';

import { COMMENT_TYPES } from '../types.js';

const GITHUB_TOKEN = process.env.AZTEC_BOT_COMMENTER_GITHUB_TOKEN;
const DOCS_PREVIEW_URL = process.env.DOCS_PREVIEW_URL;

const OWNER = 'AztecProtocol';
const REPO = 'aztec-packages';

const log = createConsoleLogger();

async function getMarkdown(prNumber: number, commentType: COMMENT_TYPES) {
  if (commentType === COMMENT_TYPES.BENCH) {
    return (await import('../benchmarks/markdown.js')).getMarkdown(prNumber);
  } else if (commentType === COMMENT_TYPES.DOCS) {
    if (!DOCS_PREVIEW_URL) {
      throw new Error('DOCS_PREVIEW_URL is not set');
    } else {
      return (await import('../docs_previews/markdown.js')).getMarkdown(DOCS_PREVIEW_URL);
    }
  } else {
    throw new Error('Invalid comment type');
  }
}

/** Function to check if a bench comment already exists */
async function getExistingComment(prNumber: number, commentType: COMMENT_TYPES) {
  try {
    const response = await sendGitHubRequest(`/repos/${OWNER}/${REPO}/issues/${prNumber}/comments`);
    const comments = JSON.parse(response);
    return comments.find((comment: any) => comment.body.includes(commentType));
  } catch (error: any) {
    throw new Error('Error checking for existing comments: ' + error.message);
  }
}

/** Function to create or update a comment */
async function upsertComment(prNumber: number, existingCommentId: string, commentType: COMMENT_TYPES) {
  try {
    const commentContent = await getMarkdown(prNumber, commentType);
    const commentData = { body: commentContent };

    const requestMethod = existingCommentId ? 'PATCH' : 'POST';
    const requestUrl = existingCommentId
      ? `/repos/${OWNER}/${REPO}/issues/comments/${existingCommentId}`
      : `/repos/${OWNER}/${REPO}/issues/${prNumber}/comments`;

    await sendGitHubRequest(requestUrl, requestMethod, commentData);
    log('Comment added or updated successfully.');
  } catch (error: any) {
    throw new Error('Error adding or updating comment: ' + error.message);
  }
}

/** Function to send a request to the GitHub API */
function sendGitHubRequest(url: string, method = 'GET', data?: object): Promise<string> {
  const apiUrl = url.startsWith('http') ? url : `https://api.github.com${url}`;
  const headers = {
    Authorization: `Bearer ${GITHUB_TOKEN}`,
    Accept: 'application/vnd.github+json',
    'X-GitHub-Api-Version': '2022-11-28',
    'User-Agent': OWNER,
    'Content-Type': undefined as string | undefined,
  };
  if (data) {
    headers['Content-Type'] = 'application/json';
  } else {
    delete headers['Content-Type'];
  }

  const requestOptions = { method, headers };

  // TODO: Use octokit instead of manually using the https node module
  return new Promise((resolve, reject) => {
    const req = https.request(apiUrl, requestOptions, res => {
      if (res.statusCode === 301 || res.statusCode === 302 || res.statusCode === 307) {
        sendGitHubRequest(res.headers.location!, method, data).then(resolve).catch(reject);
        return;
      } else {
        let data = '';
        res.on('data', chunk => {
          data += chunk;
        });

        res.on('end', () => {
          if (res.statusCode! >= 200 && res.statusCode! < 300) {
            resolve(data);
          } else {
            reject(new Error(`GitHub API request failed with ${res.statusCode}: ${data}`));
          }
        });
      }
    });

    req.on('error', error => {
      reject(error);
    });

    if (data) {
      req.write(JSON.stringify(data));
    }
    req.end();
  });
}

/** Entrypoint */
export default async function main(prNumber: number, commentType: COMMENT_TYPES = COMMENT_TYPES.BENCH) {
  const existingComment = await getExistingComment(prNumber, commentType);
  await upsertComment(prNumber, existingComment?.id, commentType);
}
