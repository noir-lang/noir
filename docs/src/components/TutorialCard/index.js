import React from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';

const Card = ({
  className,
  style,
  children,
  shadow,
  link,
}) => {
  const cardShadow = shadow ? `item shadow--${shadow}` : '';
  
  const cardContent = (
    <div className={clsx('card', className, cardShadow)} style={style}>
      {children}
    </div>
  );

  if (link) {
    return (
      <Link
        to={link}
        className="card-link-wrapper"
        style={{ textDecoration: 'none', color: 'inherit', display: 'block' }}
      >
        {cardContent}
      </Link>
    );
  }

  return cardContent;
};

export default Card;