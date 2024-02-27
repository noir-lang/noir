import { Home } from './pages/home';
import 'react-toastify/dist/ReactToastify.css';
import * as ReactDOM from 'react-dom/client';
import { ToastContainer } from 'react-toastify';

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);
root.render(
  <>
    <Home />
    <ToastContainer />
  </>,
);
