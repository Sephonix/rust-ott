# Original phase fit for SPS QuadratureShifter. Requires numpy and scipy.
# Fits (z^-2-a)/(1-a*z^-2); path B is delayed by one sample.
# This offline tool is never used by plugin builds or the audio thread.
import numpy as np
from scipy.optimize import least_squares
w=np.r_[np.geomspace(.0007,.35,400),np.linspace(.35,np.pi/2,400)]
def phases(a):
 return -2*np.arctan2((1+a[:,None])*np.sin(w),(1-a[:,None])*np.cos(w)).sum(axis=0)
def residual(x):
 a=1/(1+np.exp(-x));return phases(a[:8])-phases(a[8:])+w-np.pi/2
# Alternating log-spaced initial transition frequencies; each path has eight poles.
initial=np.array([.05,.4,.75,.9,.97,.99,.997,.9992,.2,.6,.85,.95,.985,.994,.9985,.9997])
x=np.log(initial/(1-initial))
for power in [1,2,3]:
 def f(x):
  r=residual(x);return r*np.abs(r)**(power-1)
 result=least_squares(f,x,max_nfev=5000,xtol=1e-13,ftol=1e-13,gtol=1e-13);x=result.x
 a=1/(1+np.exp(-x));print('coefficients',repr(a));print('max phase',np.abs(residual(x)).max(),'image dB',20*np.log10(np.abs(residual(x)).max()/2),flush=True)
print('Full precision coefficients:'); print('\n'.join(format(v,'.17g') for v in a))
