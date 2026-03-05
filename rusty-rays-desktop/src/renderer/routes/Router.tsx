import { HashRouter, Navigate, Route, Routes } from 'react-router';
import React from 'react';
import ROUTES from '@/routes/route-constants.ts';
import { EditorPage, LandingPage } from '@/pages';

const AppRouter: React.FC = () => {
  return (
    <HashRouter>
      <Routes>
        <Route path={ROUTES.LANDING} element={<LandingPage />} />
        <Route path={ROUTES.EDITOR} element={<EditorPage />} />
        <Route path="/" element={<Navigate replace to={ROUTES.LANDING} />} />
        <Route path="*" element={<Navigate replace to={ROUTES.LANDING} />} />
      </Routes>
    </HashRouter>
  );
};

export { AppRouter };
