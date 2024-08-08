import React from 'react';
import clsx from 'clsx';

const CardBody = ({
  className,
  style,
  children,
}) => {   
  return (
    <div
      className={clsx('card__body', className)}
      style={style}
    >
      {children}
    </div>
  );
}

export default CardBody;