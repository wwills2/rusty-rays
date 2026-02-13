import { BrowserRouter, Navigate, Route, Routes } from 'react-router';
import React from 'react';
import ROUTES from '@/routes/route-constants.ts';
import { EditorPage, LandingPage } from '@/pages';

const AppRouter: React.FC = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path={ROUTES.LANDING} element={<LandingPage />} />
        <Route path={ROUTES.EDITOR} element={<EditorPage />} />
        <Route path="/" element={<Navigate replace to={ROUTES.LANDING} />} />
        <Route path="*" element={<Navigate replace to={ROUTES.LANDING} />} />
      </Routes>
    </BrowserRouter>
  );
};

export { AppRouter };
